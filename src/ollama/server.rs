use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use sysinfo::{Pid, System};

use crate::errors::LLMeterError;
use crate::ollama::client::OllamaClient;
use crate::utils::ensure_dir;

#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub installed: bool,
    pub executable: Option<String>,
    pub running: bool,
    pub version: Option<String>,
    pub tracked_pid: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct OllamaServerManager {
    client: OllamaClient,
    pub state_dir: PathBuf,
    pid_file: PathBuf,
    log_file: PathBuf,
}

impl OllamaServerManager {
    pub fn new(client: OllamaClient, state_dir: PathBuf) -> Self {
        let pid_file = state_dir.join("ollama-server.pid.json");
        let log_file = state_dir.join("ollama-server.log");
        OllamaServerManager {
            client,
            state_dir,
            pid_file,
            log_file,
        }
    }

    pub fn executable() -> Option<String> {
        let exe_name = if cfg!(windows) { "ollama.exe" } else { "ollama" };
        std::env::var_os("PATH").and_then(|paths| {
            std::env::split_paths(&paths).find_map(|dir| {
                let full_path = dir.join(exe_name);
                if full_path.is_file() {
                    Some(full_path.to_string_lossy().to_string())
                } else {
                    None
                }
            })
        })
    }

    pub fn require_executable() -> Result<String> {
        Self::executable().ok_or_else(|| LLMeterError::OllamaNotInstalled.into())
    }

    pub fn installed_version_cli(&self) -> Option<String> {
        let exe = Self::executable()?;
        let output = Command::new(&exe).arg("--version").output().ok()?;
        let text = String::from_utf8_lossy(if !output.stdout.is_empty() {
            &output.stdout
        } else {
            &output.stderr
        })
        .trim()
        .to_string();
        if text.is_empty() { None } else { Some(text) }
    }

    fn read_pid(&self) -> Option<u32> {
        if !self.pid_file.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&self.pid_file).ok()?;
        let payload: serde_json::Value = serde_json::from_str(&content).ok()?;
        payload.get("pid").and_then(|v| v.as_u64()).map(|v| v as u32)
    }

    fn write_pid(&self, pid: u32) -> Result<()> {
        ensure_dir(&self.state_dir).context("Failed to create state directory")?;
        let payload = serde_json::json!({"pid": pid});
        let content = serde_json::to_string_pretty(&payload)?;
        std::fs::write(&self.pid_file, content).context("Failed to write PID file")?;
        Ok(())
    }

    fn clear_pid(&self) {
        let _ = std::fs::remove_file(&self.pid_file);
    }

    fn pid_alive(pid: u32) -> bool {
        if pid == 0 {
            return false;
        }
        let mut system = System::new();
        system.refresh_processes(
            sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
            true,
        );
        system.process(Pid::from_u32(pid)).is_some()
    }

    pub fn status(&self) -> ServerStatus {
        let installed = Self::executable().is_some();
        let (version, running) = match self.client.version() {
            Ok(v) => (
                v.get("version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                true,
            ),
            Err(_) => (None, false),
        };

        let mut pid = self.read_pid();
        if let Some(p) = pid {
            if !Self::pid_alive(p) {
                self.clear_pid();
                pid = None;
            }
        }

        ServerStatus {
            installed,
            executable: Self::executable(),
            running,
            version,
            tracked_pid: pid,
        }
    }

    pub fn start(&self, wait_seconds: f64) -> Result<ServerStatus> {
        if self.client.is_running() {
            return Ok(self.status());
        }

        let exe = Self::require_executable()?;
        ensure_dir(&self.state_dir).context("Failed to create state directory")?;

        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .context("Failed to open log file")?;

        let log_for_stdout = log_file
            .try_clone()
            .context("Failed to clone log file handle")?;

        let mut cmd = Command::new(&exe);
        cmd.arg("serve")
            .stdout(log_for_stdout)
            .stderr(log_file)
            .stdin(Stdio::null());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        }

        let mut child = cmd.spawn().context("Failed to start Ollama server")?;
        let pid = child.id();
        self.write_pid(pid)?;

        let deadline = Instant::now() + Duration::from_secs_f64(wait_seconds);
        let mut last_error: Option<anyhow::Error> = None;

        while Instant::now() < deadline {
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.clear_pid();
                    return Err(anyhow::anyhow!(
                        "Ollama exited early with code {}. See {}",
                        status,
                        self.log_file.display()
                    ));
                }
                Ok(None) => {}
                Err(e) => {
                    self.clear_pid();
                    return Err(e).context("Failed to check Ollama process status")?;
                }
            }

            match self.client.version() {
                Ok(_) => return Ok(self.status()),
                Err(e) => {
                    last_error = Some(e);
                }
            }

            std::thread::sleep(Duration::from_millis(250));
        }

        Err(anyhow::anyhow!(
            "Ollama did not become ready in {:.1}s. Last error: {:?}",
            wait_seconds,
            last_error
        ))
    }

    pub fn stop(&self, force: bool) -> Result<String> {
        if let Some(pid) = self.read_pid() {
            if Self::pid_alive(pid) {
                kill_process(pid, force)?;
                self.clear_pid();
                return Ok(format!("Stopped tracked Ollama server process PID {pid}."));
            }
        }

        self.clear_pid();

        if !force {
            return Ok(
                "No tracked Ollama server process found. This CLI only stops servers it started unless --force is used.".to_string(),
            );
        }

        let _exe = Self::require_executable()?;

        #[cfg(windows)]
        {
            Command::new("taskkill")
                .args(["/IM", "ollama.exe", "/F"])
                .status()
                .context("Failed to force-stop ollama.exe")?;
            return Ok("Requested forced termination of ollama.exe processes.".to_string());
        }

        #[cfg(unix)]
        {
            Command::new("pkill")
                .args(["-f", "ollama serve"])
                .status()
                .ok();
            return Ok("Requested forced termination of 'ollama serve' processes.".to_string());
        }

        #[allow(unreachable_code)]
        Ok("Stop requested.".to_string())
    }
}

fn kill_process(pid: u32, force: bool) -> Result<()> {
    #[cfg(windows)]
    {
        let mut args = vec!["/PID".to_string(), pid.to_string(), "/T".to_string()];
        if force {
            args.push("/F".to_string());
        }
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        Command::new("taskkill")
            .args(&arg_refs)
            .status()
            .context("Failed to kill process")?;
        Ok(())
    }

    #[cfg(unix)]
    {
        let signal = if force { "KILL" } else { "TERM" };
        Command::new("kill")
            .args([format!("-{signal}"), pid.to_string()])
            .status()
            .context("Failed to send signal")?;

        if force {
            let deadline = Instant::now() + Duration::from_secs(5);
            while Instant::now() < deadline {
                if !OllamaServerManager::pid_alive(pid) {
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }
        Ok(())
    }
}
