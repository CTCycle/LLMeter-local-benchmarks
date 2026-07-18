#![cfg(windows)]

use std::{thread, time::Duration};

use expectrl::{spawn, ControlCode, Expect};

fn menu_command() -> String {
    format!("\"{}\" --timeout 0.1 menu", env!("CARGO_BIN_EXE_llmeter"))
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
