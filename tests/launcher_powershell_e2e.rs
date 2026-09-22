#![cfg(windows)]

use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{Mutex, MutexGuard, OnceLock},
    time::{Duration, SystemTime},
};

use expectrl::{Expect, Session};
use tempfile::TempDir;

static PROCESS_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct ScopedEnvironment {
    _lock: MutexGuard<'static, ()>,
    previous: Vec<(&'static str, Option<OsString>)>,
}

impl ScopedEnvironment {
    fn new() -> Self {
        let lock = PROCESS_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self {
            _lock: lock,
            previous: Vec::new(),
        }
    }

    fn set(&mut self, key: &'static str, value: &OsStr) {
        self.previous.push((key, std::env::var_os(key)));
        std::env::set_var(key, value);
    }
}

impl Drop for ScopedEnvironment {
    fn drop(&mut self) {
        for (key, value) in self.previous.iter().rev() {
            if let Some(value) = value {
                std::env::set_var(key, value);
            } else {
                std::env::remove_var(key);
            }
        }
    }
}

struct LauncherFixture {
    root: TempDir,
    script: PathBuf,
    home: PathBuf,
    temp: PathBuf,
}

impl LauncherFixture {
    fn new() -> Self {
        let fixture_parent = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("QA");
        let root = TempDir::new_in(fixture_parent).expect("create isolated launcher fixture");
        let script = root.path().join("run_llmeter.ps1");
        let source_script = Path::new(env!("CARGO_MANIFEST_DIR")).join("run_llmeter.ps1");
        fs::copy(source_script, &script).expect("copy launcher into fixture");
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"launcher-fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        )
        .expect("write minimal fixture manifest");

        let home = root.path().join("home");
        let temp = root.path().join("temp");
        fs::create_dir_all(&home).expect("create fixture home");
        fs::create_dir_all(&temp).expect("create fixture temp root");

        Self {
            root,
            script,
            home,
            temp,
        }
    }

    fn default_binary(&self) -> PathBuf {
        self.root
            .path()
            .join("target")
            .join("release")
            .join("llmeter.exe")
    }

    fn fallback_target(&self) -> PathBuf {
        self.temp.join("llmeter-build")
    }

    fn install_binary(&self, destination: &Path) {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).expect("create fixture binary directory");
        }
        fs::copy(env!("CARGO_BIN_EXE_llmeter"), destination)
            .expect("copy built llmeter binary into fixture");
        fs::OpenOptions::new()
            .append(true)
            .open(destination)
            .expect("open fixture binary for timestamp update")
            .set_modified(SystemTime::now())
            .expect("refresh fixture binary timestamp");
    }

    fn owned_paths(&self, home: &Path) -> Vec<PathBuf> {
        vec![
            self.root.path().join("target"),
            self.root.path().join(".uv-cache"),
            self.fallback_target(),
            home.join("bin"),
            home.join("config"),
            home.join("benchmark_results"),
        ]
    }

    fn create_cleanup_sentinels(&self, home: &Path) -> Vec<PathBuf> {
        let owned_paths = self.owned_paths(home);
        for path in &owned_paths {
            fs::create_dir_all(path).expect("create launcher-owned sentinel directory");
            fs::write(path.join("sentinel.txt"), "owned").expect("write owned sentinel");
        }
        fs::write(self.root.path().join("keep.txt"), "repository sentinel")
            .expect("write repository sentinel");
        fs::write(home.join("keep.txt"), "home sentinel").expect("write home sentinel");
        owned_paths
    }

    fn powershell_command(&self, args: &[&str]) -> Command {
        self.powershell_command_with_home(args, &self.home)
    }

    fn powershell_command_with_home(&self, args: &[&str], home: &Path) -> Command {
        let mut command = Command::new("powershell.exe");
        command
            .arg("-NoProfile")
            .arg("-File")
            .arg(&self.script)
            .args(args)
            .env("TEMP", &self.temp)
            .env("TMP", &self.temp)
            .env("LLMETER_HOME", home);
        command
    }

    fn scoped_environment(&self, home: &Path) -> ScopedEnvironment {
        let mut environment = ScopedEnvironment::new();
        environment.set("TEMP", self.temp.as_os_str());
        environment.set("TMP", self.temp.as_os_str());
        environment.set("LLMETER_HOME", home.as_os_str());
        environment.set("LLMETER_CONPTY", OsStr::new("1"));
        environment
    }

    fn conpty_transcript_powershell_command(&self) -> Command {
        let wrapper = self.root.path().join("launcher_transcript_wrapper.ps1");
        fs::write(
            &wrapper,
            r#"Start-Transcript -Path $env:LLMETER_TRANSCRIPT -Force | Out-Null
& (Join-Path $PSScriptRoot 'run_llmeter.ps1') -Action RemoveAllData
"#,
        )
        .expect("write transcript wrapper");
        let mut command = Command::new("powershell.exe");
        command
            .arg("-NoProfile")
            .arg("-File")
            .arg(format!("\"{}\"", wrapper.display()));
        command
    }

    fn run_redirected(&self, args: &[&str]) -> Output {
        let mut command = self.powershell_command(args);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("run launcher with redirected streams")
    }

    fn output_text(output: &Output) -> String {
        format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    }

    fn create_cargo_shim(&self) -> PathBuf {
        let shim_dir = self.root.path().join("cargo-shim");
        fs::create_dir_all(&shim_dir).expect("create cargo shim directory");
        fs::write(
            shim_dir.join("cargo.cmd"),
            r#"@echo off
set "fallback_dir="
:parse
if "%~1"=="" goto finish
if /I "%~1"=="--target-dir" (
  set "fallback_dir=%~2"
  shift
)
shift
goto parse
:finish
if defined fallback_dir (
  if not exist "%fallback_dir%\release" mkdir "%fallback_dir%\release"
  copy /Y "%LLMETER_FIXTURE_BINARY%" "%fallback_dir%\release\llmeter.exe" >nul
  echo fallback build succeeded 1>&2
  exit /b 0
)
echo default build intentionally failed 1>&2
exit /b 17
"#,
        )
        .expect("write deterministic cargo shim");
        shim_dir
    }

    fn prepend_path(command: &mut Command, directory: &Path) {
        let mut paths = vec![directory.to_path_buf()];
        if let Some(existing) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&existing));
        }
        command.env(
            "PATH",
            std::env::join_paths(paths).expect("compose fixture child PATH"),
        );
    }
}

fn wait_for_transcript(path: &Path, needle: &str, timeout: Duration) -> String {
    let started = std::time::Instant::now();
    let mut output = String::new();

    loop {
        if let Ok(text) = fs::read_to_string(path) {
            output = text;
        }

        if output.contains(needle) {
            return output;
        }
        if started.elapsed() >= timeout {
            panic!("timed out waiting for {needle:?}; transcript was {output:?}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn launcher_reuses_fixture_release_binary_and_forwards_status() {
    let fixture = LauncherFixture::new();
    fixture.install_binary(&fixture.default_binary());

    let version = fixture.run_redirected(&["--version"]);
    assert_eq!(
        version.status.code(),
        Some(0),
        "{}",
        LauncherFixture::output_text(&version)
    );
    let version_text = LauncherFixture::output_text(&version);
    assert!(
        version_text.contains("Using existing release build."),
        "{version_text}"
    );
    assert!(version_text.contains("llmeter 0.4.0"), "{version_text}");

    let status = fixture.run_redirected(&[
        "--provider",
        "openai-compatible",
        "--base-url",
        "http://127.0.0.1:9",
        "--timeout",
        "1",
        "status",
    ]);
    assert_eq!(
        status.status.code(),
        Some(1),
        "{}",
        LauncherFixture::output_text(&status)
    );
    let status_text = LauncherFixture::output_text(&status);
    assert!(
        status_text.contains("http://127.0.0.1:9/v1"),
        "{status_text}"
    );
}

#[test]
fn noninteractive_clean_whatif_refuses_before_mutation() {
    let fixture = LauncherFixture::new();
    let owned_paths = fixture.create_cleanup_sentinels(&fixture.home);

    let output = fixture.run_redirected(&["-Action", "Clean", "-WhatIf"]);
    assert!(
        !output.status.success(),
        "{}",
        LauncherFixture::output_text(&output)
    );
    let text = LauncherFixture::output_text(&output);
    assert!(
        text.contains("requires an interactive console; no files were changed."),
        "{text}"
    );
    for path in owned_paths {
        assert!(
            path.exists(),
            "noninteractive refusal mutated {}",
            path.display()
        );
    }
    assert!(fixture.root.path().join("keep.txt").is_file());
    assert!(fixture.home.join("keep.txt").is_file());
}

#[test]
fn fallback_build_is_selected_after_default_build_failure() {
    let fixture = LauncherFixture::new();
    let shim_dir = fixture.create_cargo_shim();
    let mut command = fixture.powershell_command(&["--version"]);
    command.env("LLMETER_FIXTURE_BINARY", env!("CARGO_BIN_EXE_llmeter"));
    LauncherFixture::prepend_path(&mut command, &shim_dir);
    let output = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run launcher through deterministic cargo shim");

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        LauncherFixture::output_text(&output)
    );
    let text = LauncherFixture::output_text(&output);
    assert!(text.contains("Default target build failed."), "{text}");
    assert!(text.contains("Retrying with fallback target dir"), "{text}");
    assert!(text.contains("llmeter 0.4.0"), "{text}");
    assert!(
        fixture
            .fallback_target()
            .join("release")
            .join("llmeter.exe")
            .is_file(),
        "{text}"
    );
}

#[test]
fn interactive_remove_all_data_cleans_only_owned_paths() {
    let fixture = LauncherFixture::new();
    let owned_paths = fixture.create_cleanup_sentinels(&fixture.home);
    let transcript = fixture.root.path().join("cleanup-transcript.txt");
    let mut environment = fixture.scoped_environment(&fixture.home);
    environment.set("LLMETER_TRANSCRIPT", transcript.as_os_str());
    let mut session = Session::spawn(fixture.conpty_transcript_powershell_command())
        .expect("spawn interactive launcher cleanup");
    let prompt = wait_for_transcript(&transcript, "[y/N]", Duration::from_secs(5));
    session.send("y").expect("confirm isolated cleanup");
    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("wait for launcher cleanup process");

    assert_eq!(status, 0);
    let completion_text = fs::read_to_string(&transcript).expect("read cleanup transcript");
    assert!(prompt.contains("[y/N]"), "{prompt}");
    assert!(
        completion_text.contains("[DONE] Launcher-owned cleanup completed."),
        "{completion_text}"
    );
    for path in owned_paths {
        assert!(
            !path.exists(),
            "launcher-owned path survived: {}",
            path.display()
        );
    }
    assert!(fixture.root.path().join("keep.txt").is_file());
    assert!(fixture.home.join("keep.txt").is_file());
    assert!(fixture.script.is_file());
    assert!(fixture.root.path().join("Cargo.toml").is_file());
}

#[test]
fn interactive_protected_home_refuses_after_affirmative_confirmation() {
    let fixture = LauncherFixture::new();
    let protected_home = fixture.root.path().to_path_buf();
    let owned_paths = fixture.create_cleanup_sentinels(&protected_home);
    let transcript = fixture.root.path().join("protected-transcript.txt");
    let mut environment = fixture.scoped_environment(&protected_home);
    environment.set("LLMETER_TRANSCRIPT", transcript.as_os_str());
    let mut session = Session::spawn(fixture.conpty_transcript_powershell_command())
        .expect("spawn protected-home launcher cleanup");
    let prompt = wait_for_transcript(&transcript, "[y/N]", Duration::from_secs(5));
    session
        .send("y")
        .expect("confirm disposable protected-home probe");
    let status = session
        .get_process_mut()
        .wait(Some(5_000))
        .expect("wait for protected-home launcher process");

    assert_ne!(status, 0);
    let refusal_text = fs::read_to_string(&transcript).expect("read protected-home transcript");
    assert!(prompt.contains("[y/N]"), "{prompt}");
    assert!(
        refusal_text.contains("Refusing to remove broad or protected LLMeter home"),
        "{refusal_text}"
    );
    for path in owned_paths {
        assert!(
            path.exists(),
            "protected-home refusal mutated {}",
            path.display()
        );
    }
    assert!(fixture.root.path().join("keep.txt").is_file());
    assert!(fixture.home.is_dir());
    assert!(fixture.script.is_file());
    assert!(fixture.root.path().join("Cargo.toml").is_file());
}
