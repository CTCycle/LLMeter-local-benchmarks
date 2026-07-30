#![cfg(windows)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{thread, time::Duration};

use expectrl::{spawn, ControlCode, Expect};

fn menu_command() -> String {
    format!("\"{}\" --timeout 0.1 menu", env!("CARGO_BIN_EXE_llmeter"))
}

fn menu_command_with_provider(base_url: &str) -> String {
    format!(
        "\"{}\" --provider openai-compatible --base-url {base_url} --timeout 0.1 menu",
        env!("CARGO_BIN_EXE_llmeter")
    )
}

struct MockProvider {
    base_url: String,
    address: String,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockProvider {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind PTY mock provider");
        let address = listener
            .local_addr()
            .expect("PTY mock provider address")
            .to_string();
        let base_url = format!("http://{address}/v1");
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                if let Ok((mut stream, _)) = listener.accept() {
                    handle_connection(&mut stream);
                }
            }
        });
        Self {
            base_url,
            address,
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for MockProvider {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(&self.address);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn handle_connection(stream: &mut TcpStream) {
    let mut buffer = [0_u8; 4096];
    let size = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..size]);
    let (status, body) = if request.starts_with("GET /v1/models") {
        (
            "200 OK",
            r#"{"object":"list","data":[{"id":"mock-model","object":"model"}]}"#,
        )
    } else {
        ("404 Not Found", r#"{"error":{"message":"not found"}}"#)
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

#[test]
fn pty_menu_process_can_be_interrupted_without_leaking() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let mut session = spawn(menu_command()).expect("spawn llmeter in a Windows ConPTY session");
    // ConPTY launches a real raw-mode menu. `expectrl` 0.9 cannot reliably
    // match crossterm alternate-screen output on this Windows host, so visible
    // navigation assertions stay at the deterministic key-event boundary.
    thread::sleep(Duration::from_millis(500));
    session.send(ControlCode::ETX).expect("interrupt menu");
    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits");
    assert_eq!(status, 130);
}

#[test]
fn pty_menu_eof_exits_with_interrupt_code() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let mut session = spawn(menu_command()).expect("spawn llmeter in a Windows ConPTY session");
    thread::sleep(Duration::from_millis(500));
    session.send(ControlCode::EOT).expect("send EOF to menu");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits after EOF");
    assert_eq!(status, 130);
}

#[test]
fn pty_nested_selection_cancel_returns_to_menu() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let mut session = spawn(menu_command()).expect("spawn llmeter in a Windows ConPTY session");
    thread::sleep(Duration::from_millis(500));

    // Enter Provider setup, move to Set default provider, and open its nested
    // provider selection prompt. Escape must cancel that prompt and return to
    // the provider menu instead of terminating with a generic error.
    session.send("\r").expect("open provider setup");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}[B\u{1b}[B\u{1b}[B\r")
        .expect("open nested provider prompt");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}")
        .expect("cancel nested provider prompt");
    thread::sleep(Duration::from_millis(250));

    session
        .send(ControlCode::ETX)
        .expect("interrupt after nested cancel");
    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits");
    assert_eq!(status, 130);
}

#[test]
fn pty_nested_menu_back_then_clean_exit() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let mut session = spawn(menu_command()).expect("spawn llmeter in a Windows ConPTY session");
    thread::sleep(Duration::from_millis(500));

    // Enter Provider setup, use Escape for the submenu Back action, then
    // select the top-level Exit item. The process should leave cleanly.
    session.send("\r").expect("open provider setup");
    thread::sleep(Duration::from_millis(250));
    session.send("\u{1b}").expect("back from provider setup");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}[B\u{1b}[B\u{1b}[B\u{1b}[B\u{1b}[B\r")
        .expect("select top-level exit");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits cleanly");
    assert_eq!(status, 0);
}

#[test]
fn pty_pause_prompt_interrupt_returns_130() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let mut session = spawn(menu_command()).expect("spawn llmeter in a Windows ConPTY session");
    thread::sleep(Duration::from_millis(500));

    // Main menu -> Provider setup -> List supported provider presets -> the
    // pause prompt. Ctrl+C must propagate through pause() and exit 130.
    session.send("\r").expect("open provider setup");
    thread::sleep(Duration::from_millis(250));
    session.send("\u{1b}[B\r").expect("open provider catalog");
    thread::sleep(Duration::from_millis(300));
    session
        .send(ControlCode::ETX)
        .expect("interrupt pause prompt");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits after pause interruption");
    assert_eq!(status, 130);
}

#[test]
fn pty_reports_list_cancel_returns_to_menu() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let provider = MockProvider::start();
    let mut session =
        spawn(menu_command_with_provider(&provider.base_url)).expect("spawn mocked provider menu");
    thread::sleep(Duration::from_millis(500));

    // Main menu -> Reports and comparisons -> List saved files -> pause.
    // Escape cancels the pause and returns to the reports menu; Ctrl+C then
    // verifies the process can still terminate through the outer menu.
    session
        .send("\u{1b}[B\u{1b}[B\u{1b}[B\r")
        .expect("open reports menu");
    thread::sleep(Duration::from_millis(300));
    session.send("\r").expect("list report files");
    thread::sleep(Duration::from_millis(300));
    session.send("\u{1b}").expect("cancel report pause");
    thread::sleep(Duration::from_millis(300));
    session
        .send(ControlCode::ETX)
        .expect("interrupt after returning to reports menu");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits after report cancellation");
    assert_eq!(status, 130);
}

#[test]
fn pty_nested_model_selection_cancel_returns_to_menu() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let provider = MockProvider::start();
    let mut session =
        spawn(menu_command_with_provider(&provider.base_url)).expect("spawn mocked provider menu");
    thread::sleep(Duration::from_millis(500));

    // Main menu -> Model inventory -> Show raw model metadata -> model selector.
    session.send("\u{1b}[B\r").expect("open model inventory");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}[B\u{1b}[B\r")
        .expect("open model selector");
    thread::sleep(Duration::from_millis(300));
    session.send("\u{1b}").expect("cancel model selector");
    thread::sleep(Duration::from_millis(250));

    session
        .send(ControlCode::ETX)
        .expect("interrupt after model selection cancel");
    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits");
    assert_eq!(status, 130);
}

#[test]
fn pty_numeric_prompt_cancel_returns_to_menu() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let provider = MockProvider::start();
    let mut session =
        spawn(menu_command_with_provider(&provider.base_url)).expect("spawn mocked provider menu");
    thread::sleep(Duration::from_millis(500));

    // Main menu -> Benchmark workspace -> Standard LLM benchmark. Accept the
    // provider, model, and benchmark selectors, then cancel the numeric runs
    // prompt. Cancellation must return to the benchmark menu, not exit 1.
    session
        .send("\u{1b}[B\u{1b}[B\r")
        .expect("open benchmark workspace");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}[B\u{1b}[B\r")
        .expect("open standard benchmark");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept provider selector");
    thread::sleep(Duration::from_millis(500));
    session.send(" \r").expect("select all exposed models");
    thread::sleep(Duration::from_millis(250));
    session.send(" \r").expect("select all benchmarks");
    thread::sleep(Duration::from_millis(250));
    session.send("\u{1b}").expect("cancel numeric prompt");
    thread::sleep(Duration::from_millis(250));

    session
        .send(ControlCode::ETX)
        .expect("interrupt after numeric cancellation");
    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits");
    assert_eq!(status, 130);
}

#[test]
fn pty_performance_confirmation_interrupt_returns_130() {
    std::env::set_var("LLMETER_CONPTY", "1");
    let provider = MockProvider::start();
    let mut session =
        spawn(menu_command_with_provider(&provider.base_url)).expect("spawn mocked provider menu");
    thread::sleep(Duration::from_millis(500));

    // Main menu -> Benchmark workspace -> Performance benchmark. Accept each
    // guided value through the plan preview, then interrupt the confirmation
    // prompt. The nested confirmation must use the documented exit code 130.
    session
        .send("\u{1b}[B\u{1b}[B\r")
        .expect("open benchmark workspace");
    thread::sleep(Duration::from_millis(250));
    session
        .send("\u{1b}[B\r")
        .expect("open performance benchmark");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept performance provider");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept capability probe");
    thread::sleep(Duration::from_millis(500));
    session.send(" \r").expect("select all exposed models");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept performance profile");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept measured runs");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept warmup requests");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept streaming choice");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept raw export choice");
    thread::sleep(Duration::from_millis(250));
    session.send("\r").expect("accept report choice");
    thread::sleep(Duration::from_millis(500));
    session
        .send(ControlCode::ETX)
        .expect("interrupt performance confirmation");

    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("menu process exits after confirmation interruption");
    assert_eq!(status, 130);
}
