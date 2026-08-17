use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::Context;

use crate::config::llmeter_home_dir;
use crate::errors::LLMeterError;
use crate::utils;

const BIN_DIR_NAME: &str = "bin";
const CONFIG_DIR_NAME: &str = "config";
const RESULTS_DIR_NAME: &str = "benchmark_results";
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

    if purge_home {
        validate_purge_home(&paths.home_dir)?;
    }

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
    if purge_home {
        purge_owned_home(&paths)?;
    } else {
        remove_empty_or_missing_dir(&paths.bin_dir)?;
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
            "Purged LLMeter-owned data under {} (the home directory is removed only when empty).",
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
    if target.is_dir() {
        return Err(LLMeterError::InvalidOption(format!(
            "Cannot replace managed executable {}; the target is a directory.",
            target.display()
        ))
        .into());
    }
    let suffix = utils::utc_now_run_id_stamp();
    let temp_target = target.with_extension(format!("llmeter-tmp-{suffix}"));
    let backup_target = target.with_extension(format!("llmeter-backup-{suffix}"));
    fs::copy(source, &temp_target).with_context(|| {
        format!(
            "Failed to copy executable from {} to {}",
            source.display(),
            temp_target.display()
        )
    })?;
    let had_target = target.exists();
    if had_target {
        fs::rename(target, &backup_target).with_context(|| {
            format!(
                "Failed to stage the existing executable {} for replacement",
                target.display()
            )
        })?;
    }
    if let Err(error) = fs::rename(&temp_target, target) {
        let _ = remove_if_exists(&temp_target);
        if had_target && !target.exists() {
            if let Err(restore_error) = fs::rename(&backup_target, target) {
                return Err(anyhow::anyhow!(
                    "Failed to move staged executable {} into {} ({error}); restoring the previous executable also failed ({restore_error})",
                    temp_target.display(),
                    target.display()
                ));
            }
        }
        return Err(error).with_context(|| {
            format!(
                "Failed to move staged executable {} into {}",
                temp_target.display(),
                target.display()
            )
        });
    }
    if had_target {
        remove_if_exists(&backup_target).with_context(|| {
            format!(
                "Installed {} but failed to remove its replacement backup {}",
                target.display(),
                backup_target.display()
            )
        })?;
    }
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
    let backup = target.with_extension(format!("llmeter-backup-{}", utils::utc_now_run_id_stamp()));
    format!(
        "@echo off\r\nsetlocal\r\nping 127.0.0.1 -n 3 >nul\r\nset \"staged={}\"\r\nset \"target={}\"\r\nset \"backup={}\"\r\nif exist \"%target%\" move /Y \"%target%\" \"%backup%\" >nul\r\nif errorlevel 1 goto :restore\r\nmove /Y \"%staged%\" \"%target%\" >nul\r\nif errorlevel 1 goto :restore\r\nif exist \"%backup%\" del /F /Q \"%backup%\"\r\nif exist \"%staged%\" del /F /Q \"%staged%\"\r\n(goto) 2>nul & del \"%~f0\"\r\nexit /b 0\r\n:restore\r\nif exist \"%backup%\" move /Y \"%backup%\" \"%target%\" >nul\r\nif exist \"%staged%\" del /F /Q \"%staged%\"\r\nexit /b 1\r\n",
        staged_source.display(),
        target.display(),
        backup.display()
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
            "if exist \"{}\" rmdir /Q \"{}\"",
            paths.bin_dir.display(),
            paths.bin_dir.display()
        ),
    ];
    if purge_home {
        let owned_bin_dir = paths.home_dir.join(BIN_DIR_NAME);
        let config_dir = paths.home_dir.join(CONFIG_DIR_NAME);
        let results_dir = paths.home_dir.join(RESULTS_DIR_NAME);
        lines.extend([
            format!(
                "if exist \"{}\" rmdir /Q /S \"{}\"",
                owned_bin_dir.display(),
                owned_bin_dir.display()
            ),
            format!(
                "if exist \"{}\" rmdir /Q /S \"{}\"",
                config_dir.display(),
                config_dir.display()
            ),
            format!(
                "if exist \"{}\" rmdir /Q /S \"{}\"",
                results_dir.display(),
                results_dir.display()
            ),
        ]);
        lines.push(format!(
            "if exist \"{}\" rmdir /Q \"{}\"",
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

fn purge_owned_home(paths: &ManagedInstallPaths) -> anyhow::Result<()> {
    remove_dir_if_exists(&paths.home_dir.join(BIN_DIR_NAME))?;
    remove_dir_if_exists(&paths.home_dir.join(CONFIG_DIR_NAME))?;
    remove_dir_if_exists(&paths.home_dir.join(RESULTS_DIR_NAME))?;
    remove_empty_or_missing_dir(&paths.bin_dir)?;
    remove_empty_or_missing_dir(&paths.home_dir)
}

fn validate_purge_home(home_dir: &Path) -> anyhow::Result<()> {
    let candidate = absolute_path(home_dir);
    if candidate.parent().is_none() {
        return Err(LLMeterError::InvalidOption(format!(
            "Refusing to purge filesystem root {}.",
            home_dir.display()
        ))
        .into());
    }

    let mut protected = vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))];
    protected.push(std::env::temp_dir());
    if let Some(user_home) = dirs::home_dir() {
        protected.push(user_home);
    }
    if protected
        .iter()
        .map(|path| absolute_path(path))
        .any(|path| path.starts_with(&candidate))
    {
        return Err(LLMeterError::InvalidOption(format!(
            "Refusing to purge broad or protected path {}. Set LLMETER_HOME to a dedicated LLMeter directory.",
            home_dir.display()
        ))
        .into());
    }

    if candidate.exists() && !candidate.is_dir() {
        return Err(LLMeterError::InvalidOption(format!(
            "Cannot purge LLMeter home {}; it is not a directory.",
            home_dir.display()
        ))
        .into());
    }
    Ok(())
}

fn absolute_path(path: &Path) -> PathBuf {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    fs::canonicalize(&path).unwrap_or(path)
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
        build_windows_update_script, install, managed_install_paths, uninstall, update,
        validate_purge_home, ManagedInstallPaths, CONFIG_DIR_NAME, RESULTS_DIR_NAME,
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
        let config_dir = paths.home_dir.join(CONFIG_DIR_NAME);
        let results_dir = paths.home_dir.join(RESULTS_DIR_NAME);
        assert!(script.contains(&format!("rmdir /Q /S \"{}\"", config_dir.display())));
        assert!(script.contains(&format!("rmdir /Q /S \"{}\"", results_dir.display())));
        assert!(script.contains(&format!("rmdir /Q \"{}\"", paths.home_dir.display())));
        assert!(!script.contains(&format!("rmdir /Q /S \"{}\"", paths.home_dir.display())));
    }

    #[test]
    fn update_script_restores_the_previous_executable_on_failure() {
        let script = build_windows_update_script(
            PathBuf::from("C:\\Users\\tester\\AppData\\Local\\Temp\\update.exe").as_path(),
            PathBuf::from("C:\\Users\\tester\\.llmeter\\bin\\llmeter.exe").as_path(),
        );
        assert!(script.contains("move /Y \"%target%\" \"%backup%\""));
        assert!(script.contains("goto :restore"));
        assert!(script.contains(":restore"));
        assert!(script.contains("move /Y \"%backup%\" \"%target%\""));
    }

    #[test]
    fn purge_rejects_protected_broad_paths() {
        let current = std::env::current_dir().unwrap();
        assert!(validate_purge_home(&current).is_err());
        assert!(validate_purge_home(&std::env::temp_dir()).is_err());
        if let Some(home) = dirs::home_dir() {
            assert!(validate_purge_home(&home).is_err());
        }
    }

    #[test]
    fn purge_allows_a_dedicated_missing_directory() {
        let temp = tempdir().unwrap();
        let dedicated = temp.path().join("llmeter-home");
        assert!(validate_purge_home(&dedicated).is_ok());
    }

    #[test]
    fn managed_install_update_uninstall_round_trip_uses_only_owned_data() {
        let _guard = env_lock().lock().unwrap();
        let temp = tempdir().unwrap();
        let home = temp.path().join("llmeter-home");
        std::env::set_var("LLMETER_HOME", &home);

        let installed = install(None, false).unwrap();
        assert!(installed.summary.contains("Installed LLMeter"));
        let paths = managed_install_paths(None).unwrap();
        assert!(paths.exe_path.is_file());

        std::fs::create_dir_all(home.join("config")).unwrap();
        std::fs::create_dir_all(home.join("benchmark_results")).unwrap();
        std::fs::write(home.join("config").join("config.json"), b"{}").unwrap();
        std::fs::write(home.join("benchmark_results").join("result.json"), b"{}").unwrap();

        let updated = update(None, None).unwrap();
        assert!(updated.summary.contains("Updated managed install"));
        assert!(paths.exe_path.is_file());

        let removed = uninstall(None, true).unwrap();
        assert!(removed.summary.contains("Removed managed install"));
        assert!(!home.exists());

        std::env::remove_var("LLMETER_HOME");
    }
}
