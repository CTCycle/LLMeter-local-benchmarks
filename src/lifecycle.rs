use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::Context;

use crate::config::llmeter_home_dir;
use crate::errors::LLMeterError;
use crate::utils;

const BIN_DIR_NAME: &str = "bin";
const EXECUTABLE_NAME: &str = "llmeter.exe";
const CMD_LAUNCHER_NAME: &str = "llmeter.cmd";
const POWERSHELL_LAUNCHER_NAME: &str = "llmeter.ps1";

#[derive(Debug, Clone)]
pub struct ManagedInstallPaths {
    pub home_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub exe_path: PathBuf,
    pub cmd_path: PathBuf,
    pub powershell_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LifecycleMessage {
    pub summary: String,
    pub details: Vec<String>,
}

pub fn install(bin_dir_override: Option<&str>, force: bool) -> anyhow::Result<LifecycleMessage> {
    let current_exe = current_executable()?;
    let paths = managed_install_paths(bin_dir_override)?;
    utils::ensure_dir(&paths.bin_dir).with_context(|| {
        format!(
            "Failed to create managed install directory {}",
            paths.bin_dir.display()
        )
    })?;

    let same_target = same_path(&current_exe, &paths.exe_path);
    if paths.exe_path.exists() && !same_target && !force {
        return Err(LLMeterError::InvalidOption(format!(
            "Managed install already exists at {}. Re-run with --force to overwrite it.",
            paths.exe_path.display()
        ))
        .into());
    }

    if !same_target {
        copy_executable(&current_exe, &paths.exe_path)?;
    }
    write_launchers(&paths)?;

    let mut details = vec![format!("Executable: {}", paths.exe_path.display())];
    if cfg!(windows) {
        details.push(format!("CMD launcher: {}", paths.cmd_path.display()));
        details.push(format!(
            "PowerShell launcher: {}",
            paths.powershell_path.display()
        ));
    } else {
        details.push("The executable itself is the Unix shell command; no Windows launcher files were created.".to_string());
    }
    details.push(format!(
        "Add {} to PATH to run `llmeter` from new shells.",
        paths.bin_dir.display()
    ));

    Ok(LifecycleMessage {
        summary: format!("Installed LLMeter into {}", paths.bin_dir.display()),
        details,
    })
}

pub fn update(
    bin_dir_override: Option<&str>,
    source_override: Option<&str>,
) -> anyhow::Result<LifecycleMessage> {
    let current_exe = current_executable()?;
    let paths = managed_install_paths(bin_dir_override)?;
    if !paths.exe_path.exists() {
        return Err(LLMeterError::InvalidOption(format!(
            "No managed install was found at {}. Run `llmeter install` first.",
            paths.exe_path.display()
        ))
        .into());
    }

    let source_path = match source_override {
        Some(path) => PathBuf::from(path),
        None => current_exe.clone(),
    };

    if !source_path.exists() {
        return Err(LLMeterError::InvalidOption(format!(
            "Update source {} does not exist.",
            source_path.display()
        ))
        .into());
    }

    let target_is_current_exe = same_path(&current_exe, &paths.exe_path);
    let source_is_target = same_path(&source_path, &paths.exe_path);

    if source_is_target {
        return Err(LLMeterError::InvalidOption(
            "The update source points at the current managed install. Run a newer llmeter binary with `update`, or pass --source <path-to-llmeter-binary>.".to_string(),
        )
        .into());
    }

    write_launchers(&paths)?;

    if cfg!(windows) && target_is_current_exe {
        let staged_source = stage_temp_copy(&source_path, "update")?;
        let script = build_windows_update_script(&staged_source, &paths.exe_path);
        spawn_windows_script("llmeter-update", &script)?;
        return Ok(LifecycleMessage {
            summary: format!(
                "Update helper launched for {}",
                paths.exe_path.display()
            ),
            details: vec![
                format!("Replacement source: {}", source_path.display()),
                "The helper will copy the executable after this process exits; verify the installed version with `llmeter --version`.".to_string(),
            ],
        });
    }

    copy_executable(&source_path, &paths.exe_path)?;
    Ok(LifecycleMessage {
        summary: format!("Updated managed install at {}", paths.exe_path.display()),
        details: vec![format!("Replacement source: {}", source_path.display())],
    })
}

pub fn uninstall(
    bin_dir_override: Option<&str>,
    purge_home: bool,
) -> anyhow::Result<LifecycleMessage> {
    let current_exe = current_executable()?;
    let paths = managed_install_paths(bin_dir_override)?;

    if !paths.exe_path.exists() && !paths.cmd_path.exists() && !paths.powershell_path.exists() {
        return Err(LLMeterError::InvalidOption(format!(
            "No managed install was found under {}.",
            paths.bin_dir.display()
        ))
        .into());
    }

    let target_is_current_exe = same_path(&current_exe, &paths.exe_path);
    if cfg!(windows) && target_is_current_exe {
        let script = build_windows_uninstall_script(&paths, purge_home);
        spawn_windows_script("llmeter-uninstall", &script)?;
        return Ok(LifecycleMessage {
            summary: format!("Scheduled uninstall of {}", paths.bin_dir.display()),
            details: uninstall_details(&paths, purge_home),
        });
    }

    remove_if_exists(&paths.cmd_path)?;
    remove_if_exists(&paths.powershell_path)?;
    remove_if_exists(&paths.exe_path)?;
    remove_empty_or_missing_dir(&paths.bin_dir)?;
    if purge_home {
        remove_dir_if_exists(&paths.home_dir)?;
    }

    Ok(LifecycleMessage {
        summary: format!("Removed managed install from {}", paths.bin_dir.display()),
        details: uninstall_details(&paths, purge_home),
    })
}

fn uninstall_details(paths: &ManagedInstallPaths, purge_home: bool) -> Vec<String> {
    let mut details = vec![format!("Removed executable: {}", paths.exe_path.display())];
    if cfg!(windows) {
        details.push(format!(
            "Removed launchers from: {}",
            paths.bin_dir.display()
        ));
    } else {
        details.push("No Windows launcher files were present on this platform.".to_string());
    }
    if purge_home {
        details.push(format!(
            "Removed LLMeter home: {}",
            paths.home_dir.display()
        ));
    } else {
        details.push(format!(
            "Preserved LLMeter home data under {}",
            paths.home_dir.display()
        ));
    }
    details
}

fn managed_install_paths(bin_dir_override: Option<&str>) -> anyhow::Result<ManagedInstallPaths> {
    let home_dir = llmeter_home_dir().ok_or_else(|| {
        LLMeterError::Io("Unable to determine LLMeter home directory.".to_string())
    })?;
    let bin_dir = bin_dir_override
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir.join(BIN_DIR_NAME));
    let exe_path = bin_dir.join(executable_file_name());
    let cmd_path = bin_dir.join(CMD_LAUNCHER_NAME);
    let powershell_path = bin_dir.join(POWERSHELL_LAUNCHER_NAME);
    Ok(ManagedInstallPaths {
        home_dir,
        bin_dir,
        exe_path,
        cmd_path,
        powershell_path,
    })
}

fn current_executable() -> anyhow::Result<PathBuf> {
    std::env::current_exe().context("Failed to resolve the current llmeter executable path")
}

fn executable_file_name() -> &'static str {
    if cfg!(windows) {
        EXECUTABLE_NAME
    } else {
        "llmeter"
    }
}

fn copy_executable(source: &Path, target: &Path) -> anyhow::Result<()> {
    if let Some(parent) = target.parent() {
        utils::ensure_dir(parent).with_context(|| {
            format!("Failed to create target directory for {}", target.display())
        })?;
    }
    let temp_target = target.with_extension("tmp");
    fs::copy(source, &temp_target).with_context(|| {
        format!(
            "Failed to copy executable from {} to {}",
            source.display(),
            temp_target.display()
        )
    })?;
    if target.exists() {
        fs::remove_file(target).with_context(|| {
            format!("Failed to remove existing executable {}", target.display())
        })?;
    }
    fs::rename(&temp_target, target).with_context(|| {
        format!(
            "Failed to move staged executable {} into {}",
            temp_target.display(),
            target.display()
        )
    })?;
    Ok(())
}

fn write_launchers(paths: &ManagedInstallPaths) -> anyhow::Result<()> {
    if cfg!(windows) {
        fs::write(&paths.cmd_path, build_cmd_launcher(&paths.exe_path)).with_context(|| {
            format!("Failed to write CMD launcher {}", paths.cmd_path.display())
        })?;
        fs::write(
            &paths.powershell_path,
            build_powershell_launcher(&paths.exe_path),
        )
        .with_context(|| {
            format!(
                "Failed to write PowerShell launcher {}",
                paths.powershell_path.display()
            )
        })?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(&paths.exe_path).with_context(|| {
            format!(
                "Failed to inspect installed executable {}",
                paths.exe_path.display()
            )
        })?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(&paths.exe_path, permissions).with_context(|| {
            format!(
                "Failed to make installed executable runnable {}",
                paths.exe_path.display()
            )
        })?;
    }
    Ok(())
}

fn build_cmd_launcher(exe_path: &Path) -> String {
    format!("@echo off\r\n\"{}\" %*\r\n", exe_path.display())
}

fn build_powershell_launcher(exe_path: &Path) -> String {
    format!(
        "param(\r\n    [Parameter(ValueFromRemainingArguments = $true)]\r\n    [string[]]$LlmeterArgs\r\n)\r\n\r\n& \"{}\" @LlmeterArgs\r\nexit $LASTEXITCODE\r\n",
        exe_path.display()
    )
}

fn stage_temp_copy(source: &Path, prefix: &str) -> anyhow::Result<PathBuf> {
    let file_name = format!(
        "{prefix}-{}-{}",
        utils::utc_now_run_id_stamp(),
        executable_file_name()
    );
    let staged = std::env::temp_dir().join(file_name);
    fs::copy(source, &staged).with_context(|| {
        format!(
            "Failed to stage update source {} into {}",
            source.display(),
            staged.display()
        )
    })?;
    Ok(staged)
}

fn build_windows_update_script(staged_source: &Path, target: &Path) -> String {
    format!(
        "@echo off\r\nsetlocal\r\nping 127.0.0.1 -n 3 >nul\r\ncopy /Y \"{}\" \"{}\" >nul\r\nif exist \"{}\" del /F /Q \"{}\"\r\n(goto) 2>nul & del \"%~f0\"\r\n",
        staged_source.display(),
        target.display(),
        staged_source.display(),
        staged_source.display()
    )
}

fn build_windows_uninstall_script(paths: &ManagedInstallPaths, purge_home: bool) -> String {
    let mut lines = vec![
        "@echo off".to_string(),
        "setlocal".to_string(),
        "ping 127.0.0.1 -n 3 >nul".to_string(),
        format!(
            "if exist \"{}\" del /F /Q \"{}\"",
            paths.cmd_path.display(),
            paths.cmd_path.display()
        ),
        format!(
            "if exist \"{}\" del /F /Q \"{}\"",
            paths.powershell_path.display(),
            paths.powershell_path.display()
        ),
        format!(
            "if exist \"{}\" del /F /Q \"{}\"",
            paths.exe_path.display(),
            paths.exe_path.display()
        ),
        format!(
            "if exist \"{}\" rmdir /Q /S \"{}\"",
            paths.bin_dir.display(),
            paths.bin_dir.display()
        ),
    ];
    if purge_home {
        lines.push(format!(
            "if exist \"{}\" rmdir /Q /S \"{}\"",
            paths.home_dir.display(),
            paths.home_dir.display()
        ));
    }
    lines.push("(goto) 2>nul & del \"%~f0\"".to_string());
    lines.join("\r\n") + "\r\n"
}

fn spawn_windows_script(prefix: &str, script: &str) -> anyhow::Result<()> {
    let script_path =
        std::env::temp_dir().join(format!("{prefix}-{}.cmd", utils::utc_now_run_id_stamp()));
    fs::write(&script_path, script)
        .with_context(|| format!("Failed to write helper script {}", script_path.display()))?;

    let start_arg = OsString::from("start");
    let empty_title = OsString::from("");
    let background = OsString::from("/B");
    Command::new("cmd")
        .args([
            OsString::from("/C"),
            start_arg,
            empty_title,
            background,
            script_path.as_os_str().to_os_string(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to launch helper script {}", script_path.display()))?;
    Ok(())
}

fn remove_if_exists(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("Failed to remove file {}", path.display()))?;
    }
    Ok(())
}

fn remove_dir_if_exists(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove directory {}", path.display()))?;
    }
    Ok(())
}

fn remove_empty_or_missing_dir(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    match fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("Failed to remove directory {}", path.display()))
        }
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => left == right,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_cmd_launcher, build_powershell_launcher, build_windows_uninstall_script,
        managed_install_paths, ManagedInstallPaths,
    };
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    use tempfile::tempdir;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn managed_install_defaults_to_llmeter_home_bin() {
        let _guard = env_lock().lock().unwrap();
        let temp = tempdir().unwrap();
        std::env::set_var("LLMETER_HOME", temp.path());

        let paths = managed_install_paths(None).unwrap();

        assert_eq!(paths.bin_dir, temp.path().join("bin"));
        assert_eq!(paths.cmd_path, temp.path().join("bin").join("llmeter.cmd"));

        std::env::remove_var("LLMETER_HOME");
    }

    #[test]
    fn cmd_launcher_targets_the_managed_executable() {
        let launcher = build_cmd_launcher(PathBuf::from("C:\\tools\\llmeter.exe").as_path());
        assert!(launcher.contains("\"C:\\tools\\llmeter.exe\" %*"));
    }

    #[test]
    fn powershell_launcher_targets_the_managed_executable() {
        let launcher = build_powershell_launcher(PathBuf::from("C:\\tools\\llmeter.exe").as_path());
        assert!(launcher.contains("& \"C:\\tools\\llmeter.exe\" @LlmeterArgs"));
    }

    #[test]
    fn uninstall_script_can_purge_home() {
        let paths = ManagedInstallPaths {
            home_dir: PathBuf::from("C:\\Users\\tester\\.llmeter"),
            bin_dir: PathBuf::from("C:\\Users\\tester\\.llmeter\\bin"),
            exe_path: PathBuf::from("C:\\Users\\tester\\.llmeter\\bin\\llmeter.exe"),
            cmd_path: PathBuf::from("C:\\Users\\tester\\.llmeter\\bin\\llmeter.cmd"),
            powershell_path: PathBuf::from("C:\\Users\\tester\\.llmeter\\bin\\llmeter.ps1"),
        };
        let script = build_windows_uninstall_script(&paths, true);
        assert!(script.contains("rmdir /Q /S \"C:\\Users\\tester\\.llmeter\""));
    }
}
