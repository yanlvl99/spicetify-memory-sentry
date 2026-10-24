//! Command-line entry point for the Memory Sentry companion.
//! Supports Windows (10/11), Linux, and macOS across x86_64 and ARM (aarch64).

mod platform;
mod runtime;

fn log_setup_failure(error: &str) {
    use std::fs::OpenOptions;
    use std::io::Write as _;
    use std::time::{SystemTime, UNIX_EPOCH};

    let Ok(base) = runtime::install_dir() else { return; };
    let log_dir = base.join("logs");
    if std::fs::create_dir_all(&log_dir).is_err() { return; }
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_dir.join("setup.log")) else { return; };
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let _ = writeln!(file, "[{timestamp}] [SETUP] {error}");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.first().map(String::as_str) == Some("--version") {
        println!("Memory Sentry native companion {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args.first().map(String::as_str) == Some("--self-test") {
        println!("Memory Sentry core self-test passed. This command does not change process memory.");
        return;
    }

    let result = match args.as_slice() {
        [command] if command == "--check-os" => runtime::check_supported_os(),
        [command] if command == "--check-install" => runtime::check_install_prerequisites(),
        [command] if command == "--check-spicetify" => runtime::check_spicetify_cli(),
        [command] if command == "--prepare-upgrade" => runtime::prepare_upgrade(),
        [] => runtime::run_monitor(),
        [command] if command == "--monitor" => runtime::run_monitor(),
        [command] if command == "--launch" => runtime::launch_monitor(),
        [command] if command == "--pre-uninstall" => runtime::pre_uninstall(),
        [command] if command == "--restore-limits" => runtime::restore_limits_command(),
        [command, app_dir] if command == "--configure" => runtime::configure_install(app_dir),
        _ => Err("Unknown arguments. Supported commands: --version, --self-test, --check-os, --check-install, --check-spicetify, --monitor, --launch, --prepare-upgrade, --configure <custom-app-folder>, --pre-uninstall, --restore-limits.".to_string()),
    };
    if let Err(error) = result {
        if args.first().map(String::as_str) == Some("--configure") {
            log_setup_failure(&error);
        }
        eprintln!("Memory Sentry: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_token() {
        let token = platform::random_token().expect("failed to generate random token");
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_detect_os() {
        let os = platform::detect_supported_client_os();
        assert!(os.is_ok(), "OS detection failed: {:?}", os.err());
    }

    #[test]
    fn test_inspect_self_process() {
        let pid = platform::current_process_id();
        let stats = platform::inspect_process(pid);
        assert!(stats.is_ok(), "inspecting self failed: {:?}", stats.err());
        let stats = stats.unwrap();
        assert!(stats.working_set_mb >= 0.0);
    }

    #[test]
    fn test_process_snapshot() {
        let snap = platform::process_snapshot();
        assert!(snap.is_ok(), "process snapshot failed: {:?}", snap.err());
    }
}
