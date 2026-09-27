//! Testes do wrapper de scripts acionáveis (D184/D185).

use super::*;

#[test]
fn sha256_matches_known_vector() {
    // Vetor conhecido: SHA-256 de "abc".
    let digest = sha256_hex(b"abc");
    assert_eq!(
        digest,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn source_kind_and_reference() {
    let embedded = Source::Embedded {
        name: "knudge-idle.sh",
        body: "",
    };
    assert_eq!(embedded.kind(), "embedded");
    assert_eq!(embedded.reference(), "scripts/knudge-idle.sh");
    let local = Source::Local(PathBuf::from("/tmp/x.sh"));
    assert_eq!(local.kind(), "local");
    let remote = Source::Remote {
        url: "https://example.invalid/x.sh".to_string(),
        sha256: None,
    };
    assert_eq!(remote.kind(), "download");
    assert_eq!(remote.reference(), "https://example.invalid/x.sh");
}

#[test]
fn shell_spec_bash_on_unix() {
    for os in ["linux", "macos", "freebsd"] {
        assert_eq!(
            shell_spec(Path::new("/tmp/x.sh"), os).ok(),
            Some(("bash".to_string(), vec!["/tmp/x.sh".to_string()])),
            "{os}"
        );
    }
}

#[test]
fn shell_spec_powershell_on_windows() {
    assert_eq!(
        shell_spec(Path::new("C:\\worker.ps1"), "windows").ok(),
        Some((
            "powershell".to_string(),
            vec![
                "-NoProfile".to_string(),
                "-File".to_string(),
                "C:\\worker.ps1".to_string(),
            ],
        ))
    );
}

#[test]
fn shell_spec_rejects_unix_script_on_windows() {
    assert!(shell_spec(Path::new("C:\\worker.sh"), "windows").is_err());
}

#[test]
fn is_powershell_is_case_insensitive() {
    assert!(is_powershell(Path::new("a.ps1")));
    assert!(is_powershell(Path::new("a.PS1")));
    assert!(!is_powershell(Path::new("a.sh")));
    assert!(!is_powershell(Path::new("a")));
}

#[test]
fn plan_command_uses_shell_of_current_os() {
    let command = plan_command("scripts/knudge-idle.sh", &["--install".to_string()]);
    if std::env::consts::OS == "windows" {
        assert!(command.contains("manual"), "windows: {command}");
    } else {
        assert!(command.starts_with("bash "), "unix: {command}");
        assert!(command.ends_with("--install"), "unix: {command}");
    }
}

#[cfg(unix)]
#[test]
fn run_body_pipes_stdin_and_captures_stdout() {
    let run = run_body("printf 'ok\\n'", &[]);
    assert_eq!(
        run.ok().map(|run| run.stdout.trim().to_string()),
        Some("ok".to_string())
    );
}

#[cfg(unix)]
#[test]
fn run_body_propagates_failure() {
    assert!(run_body("exit 3", &[]).is_err());
}

#[cfg(unix)]
#[test]
fn run_body_passes_args_to_script() {
    let run = run_body("printf '%s' \"$1\"", &["status".to_string()]);
    assert_eq!(
        run.ok().map(|run| run.stdout.trim().to_string()),
        Some("status".to_string())
    );
}
