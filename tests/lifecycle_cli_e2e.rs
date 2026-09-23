#![cfg(windows)]

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
    thread,
    time::{Duration, Instant},
};

use tempfile::TempDir;

struct LifecycleHarness {
    root: TempDir,
    home: PathBuf,
    temp: PathBuf,
}

impl LifecycleHarness {
    fn new() -> Self {
        let qa = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("QA");
        let root = tempfile::Builder::new()
            .prefix("t1-07-lifecycle-cli-")
            .tempdir_in(qa)
            .expect("create isolated lifecycle fixture");
        let home = root.path().join("home");
        let temp = root.path().join("temp");
        fs::create_dir_all(&home).expect("create isolated LLMeter home");
        fs::create_dir_all(&temp).expect("create isolated helper temp directory");
        Self { root, home, temp }
    }

    fn run(&self, executable: &Path, args: &[&str]) -> Output {
        Command::new(executable)
            .args(["--provider", "ollama"])
            .args(args)
            .env("LLMETER_HOME", &self.home)
            .env("TEMP", &self.temp)
            .env("TMP", &self.temp)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("run LLMeter CLI")
    }

    fn wait_for(&self, timeout: Duration, description: &str, ready: impl Fn() -> bool) {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if ready() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("timed out waiting for {description}");
    }

    fn no_update_helpers(&self) -> bool {
        fs::read_dir(&self.temp)
            .map(|entries| {
                entries.filter_map(Result::ok).all(|entry| {
                    !entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("llmeter-update-")
                        && !entry.file_name().to_string_lossy().starts_with("update-")
                })
            })
            .unwrap_or(false)
    }
}

fn output_text(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn windows_managed_lifecycle_cli_installs_updates_rolls_back_and_purges_in_isolation() {
    let harness = LifecycleHarness::new();
    let source = PathBuf::from(env!("CARGO_BIN_EXE_llmeter"));
    let managed_dir = harness.home.join("bin");
    let managed = managed_dir.join("llmeter.exe");

    let installed = harness.run(&source, &["install"]);
    assert!(installed.status.success(), "{}", output_text(&installed));
    assert_eq!(fs::read(&managed).unwrap(), fs::read(&source).unwrap());
    assert!(managed_dir.join("llmeter.cmd").is_file());
    assert!(managed_dir.join("llmeter.ps1").is_file());
    assert!(harness.run(&managed, &["--version"]).status.success());

    fs::create_dir_all(harness.home.join("config")).unwrap();
    fs::create_dir_all(harness.home.join("benchmark_results")).unwrap();
    fs::write(harness.home.join("config/config.json"), b"{}\n").unwrap();
    fs::write(
        harness.home.join("benchmark_results/result.json"),
        b"fixture\n",
    )
    .unwrap();
    fs::write(harness.home.join("keep.txt"), b"user-owned sentinel\n").unwrap();

    let original = fs::read(&managed).unwrap();
    let duplicate_install = harness.run(&source, &["install"]);
    assert_eq!(duplicate_install.status.code(), Some(2));
    assert!(output_text(&duplicate_install).contains("already exists"));
    assert_eq!(fs::read(&managed).unwrap(), original);

    let replacement = harness.root.path().join("replacement.exe");
    fs::copy(&source, &replacement).unwrap();
    fs::OpenOptions::new()
        .append(true)
        .open(&replacement)
        .unwrap()
        .write_all(b"t1-07 replacement overlay\n")
        .unwrap();
    let replacement_bytes = fs::read(&replacement).unwrap();

    let stage_dir = harness.temp.clone();
    let expected_stage_len = replacement_bytes.len() as u64;
    let stage_remover = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if let Ok(entries) = fs::read_dir(&stage_dir) {
                if let Some(path) = entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .find(|path| {
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .is_some_and(|name| {
                                name.starts_with("update-") && name.ends_with("llmeter.exe")
                            })
                            && fs::metadata(path)
                                .is_ok_and(|metadata| metadata.len() == expected_stage_len)
                    })
                {
                    let remove_deadline = Instant::now() + Duration::from_secs(1);
                    loop {
                        match fs::remove_file(&path) {
                            Ok(()) => return path,
                            Err(error) if !path.exists() => {
                                panic!("update helper consumed the staged file before rollback injection: {error}");
                            }
                            Err(error) if Instant::now() < remove_deadline => {
                                let _ = error;
                                thread::sleep(Duration::from_millis(10));
                            }
                            Err(error) => {
                                panic!("remove staged update before helper runs: {error}")
                            }
                        }
                    }
                }
            }
            assert!(
                Instant::now() < deadline,
                "update helper did not stage its replacement"
            );
            thread::sleep(Duration::from_millis(5));
        }
    });
    let rollback = harness.run(
        &managed,
        &["update", "--source", replacement.to_str().unwrap()],
    );
    let rollback_text = output_text(&rollback);
    assert!(rollback.status.success(), "{rollback_text}");
    assert!(
        rollback_text.contains("Update helper launched"),
        "{rollback_text}"
    );
    assert!(stage_remover.join().is_ok(), "stage-removal watcher failed");
    harness.wait_for(Duration::from_secs(8), "rollback helper cleanup", || {
        fs::read(&managed).is_ok_and(|bytes| bytes == original) && harness.no_update_helpers()
    });
    assert!(harness.run(&managed, &["--version"]).status.success());
    assert!(fs::read_dir(&managed_dir)
        .unwrap()
        .filter_map(Result::ok)
        .all(|entry| !entry.file_name().to_string_lossy().contains("backup")));

    let updated = harness.run(
        &managed,
        &["update", "--source", replacement.to_str().unwrap()],
    );
    assert!(updated.status.success(), "{}", output_text(&updated));
    harness.wait_for(Duration::from_secs(8), "successful managed update", || {
        fs::read(&managed).is_ok_and(|bytes| bytes == replacement_bytes)
            && harness.no_update_helpers()
    });
    assert!(harness.run(&managed, &["--version"]).status.success());
    assert!(fs::read_dir(&managed_dir)
        .unwrap()
        .filter_map(Result::ok)
        .all(|entry| !entry.file_name().to_string_lossy().contains("backup")));

    let uninstalled = harness.run(&managed, &["uninstall", "--purge-home"]);
    assert!(
        uninstalled.status.success(),
        "{}",
        output_text(&uninstalled)
    );
    harness.wait_for(
        Duration::from_secs(8),
        "self-uninstall and home purge",
        || {
            !managed_dir.exists()
                && !harness.home.join("config").exists()
                && !harness.home.join("benchmark_results").exists()
        },
    );
    assert_eq!(
        fs::read(harness.home.join("keep.txt")).unwrap(),
        b"user-owned sentinel\n"
    );
    assert!(harness.root.path().join("replacement.exe").is_file());
    assert!(harness.temp.is_dir());
}
