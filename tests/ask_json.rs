use assert_cmd::Command;
use predicates::prelude::*;

fn present() -> Command {
    Command::cargo_bin("present").unwrap()
}

#[test]
fn json_happy_path_force_pick_returns_first_option() {
    let mut cmd = present();
    cmd.env("PRESENT_AUTO_PICK", "1")
        .arg("ask")
        .arg("--json")
        .write_stdin(r#"{"message":"pick","options":["a","b","c"]}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""selected":["a"]"#));
}

#[test]
fn json_empty_options_is_rejected_with_a_named_reason() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--json")
        .write_stdin(r#"{"message":"x","options":[]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("options is empty"));
}

#[test]
fn json_single_option_is_rejected_as_nothing_to_ask() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--json")
        .write_stdin(r#"{"message":"x","options":["only"]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("only one option"));
}

#[test]
fn json_empty_message_is_rejected() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--json")
        .write_stdin(r#"{"message":"","options":["a","b"]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("message is empty"));
}

#[test]
fn json_malformed_input_names_the_parse_error() {
    let mut cmd = present();
    cmd.arg("ask").arg("--json").write_stdin("{not json");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("not a valid ask request"));
}

#[test]
fn json_empty_stdin_is_rejected() {
    let mut cmd = present();
    cmd.arg("ask").arg("--json").write_stdin("");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("no input on stdin"));
}

#[test]
fn json_missing_field_is_named() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--json")
        .write_stdin(r#"{"options":["a","b"]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("missing field `message`"));
}

#[test]
fn json_non_string_option_is_rejected() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--json")
        .write_stdin(r#"{"message":"x","options":[1,2]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("expected a string"));
}

#[test]
fn json_hundred_options_force_pick_still_returns_first() {
    let mut opts = String::from("[");
    for i in 1..=100 {
        if i > 1 {
            opts.push(',');
        }
        opts.push_str(&format!("\"o{i}\""));
    }
    opts.push(']');
    let body = format!(r#"{{"message":"pick","options":{opts}}}"#);
    let mut cmd = present();
    cmd.env("PRESENT_AUTO_PICK", "1")
        .arg("ask")
        .arg("--json")
        .write_stdin(body);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""selected":["o1"]"#));
}

#[test]
fn json_multiple_flag_serializes_an_array_of_one_when_force_picked() {
    let mut cmd = present();
    cmd.env("PRESENT_AUTO_PICK", "1")
        .arg("ask")
        .arg("--json")
        .write_stdin(r#"{"message":"pick","options":["a","b"],"multiple":true}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""selected":["a"]"#));
}

#[test]
fn json_without_force_pick_and_without_tty_errors_with_a_next_step() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--json")
        .write_stdin(r#"{"message":"pick","options":["a","b"]}"#);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("PRESENT_AUTO_PICK"));
}
