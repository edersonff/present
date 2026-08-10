use assert_cmd::Command;
use predicates::prelude::*;

fn present() -> Command {
    Command::cargo_bin("present").unwrap()
}

#[test]
fn flags_json_auto_pick_returns_first_option_as_choice() {
    let mut cmd = present();
    cmd.env("PRESENT_AUTO_PICK", "1")
        .arg("--ask")
        .arg("pick")
        .arg("--options")
        .arg(r#"["a","b","c"]"#)
        .arg("--json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""choice":"a""#));
}

#[test]
fn stdin_json_auto_pick_returns_first_option_as_choice() {
    let mut cmd = present();
    cmd.env("PRESENT_AUTO_PICK", "1")
        .arg("--json")
        .write_stdin(r#"{"message":"pick","options":["a","b","c"]}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""choice":"a""#));
}

#[test]
fn stdin_json_empty_options_is_rejected() {
    let mut cmd = present();
    cmd.arg("--json")
        .write_stdin(r#"{"message":"x","options":[]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("options is empty"));
}

#[test]
fn stdin_json_single_option_is_rejected() {
    let mut cmd = present();
    cmd.arg("--json")
        .write_stdin(r#"{"message":"x","options":["only"]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("only one option"));
}

#[test]
fn stdin_json_empty_message_is_rejected() {
    let mut cmd = present();
    cmd.arg("--json")
        .write_stdin(r#"{"message":"","options":["a","b"]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("message is empty"));
}

#[test]
fn stdin_json_malformed_names_the_parse_error() {
    let mut cmd = present();
    cmd.arg("--json").write_stdin("{not json");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("not a valid ask request"));
}

#[test]
fn stdin_json_empty_stdin_is_rejected() {
    let mut cmd = present();
    cmd.arg("--json").write_stdin("");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("no input on stdin"));
}

#[test]
fn stdin_json_missing_field_is_named() {
    let mut cmd = present();
    cmd.arg("--json")
        .write_stdin(r#"{"options":["a","b"]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("missing field `message`"));
}

#[test]
fn stdin_json_non_string_option_is_rejected() {
    let mut cmd = present();
    cmd.arg("--json")
        .write_stdin(r#"{"message":"x","options":[1,2]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("expected a string"));
}

#[test]
fn flags_json_hundred_options_auto_pick_still_returns_first() {
    let mut opts = String::from("[");
    for i in 1..=100 {
        if i > 1 {
            opts.push(',');
        }
        opts.push_str(&format!("\"o{i}\""));
    }
    opts.push(']');
    let mut cmd = present();
    cmd.env("PRESENT_AUTO_PICK", "1")
        .arg("--ask")
        .arg("pick")
        .arg("--options")
        .arg(opts)
        .arg("--json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""choice":"o1""#));
}

#[test]
fn flags_json_without_auto_pick_and_without_tty_errors_with_next_step() {
    let mut cmd = present();
    cmd.arg("--ask")
        .arg("pick")
        .arg("--options")
        .arg(r#"["a","b"]"#)
        .arg("--json");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("PRESENT_AUTO_PICK"));
}
