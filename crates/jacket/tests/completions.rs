//! `jacket completions`: a script for jacket's own commands, or for the flags of `jaic`.
use std::process::Command;

fn completions(args: &[&str]) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_jacket"))
        .arg("completions")
        .args(args)
        .output()
        .unwrap();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn a_script_for_jacket_names_its_commands() {
    for (shell, expected) in [
        ("bash", "_jacket"),
        ("zsh", "#compdef jacket"),
        ("fish", "complete -c jacket"),
    ] {
        let (code, script, _) = completions(&[shell]);
        assert_eq!(code, Some(0), "{shell}");
        assert!(script.contains(expected), "{shell}: {script}");
        assert!(script.contains("migrate"), "{shell}");
    }
}

#[test]
fn a_script_for_jaic_completes_its_single_dash_flags() {
    for (shell, expected) in [
        ("bash", "complete -o filenames -F _jaic jaic"),
        ("zsh", "#compdef jaic"),
        ("fish", "complete -c jaic -o interp"),
    ] {
        let (code, script, _) = completions(&[shell, "--tool", "jaic"]);
        assert_eq!(code, Some(0), "{shell}");
        assert!(script.contains(expected), "{shell}: {script}");
    }
}

#[test]
fn an_unknown_shell_is_a_usage_error() {
    let (code, _, stderr) = completions(&["csh"]);
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains("bash") && stderr.contains("fish"),
        "{stderr}"
    );
}
