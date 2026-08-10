use assert_cmd::Command;
use predicates::prelude::*;

fn present() -> Command {
    Command::cargo_bin("present").unwrap()
}

#[test]
fn cli_pick_by_number_prints_selected_to_stdout() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["a","b","c"]"#)
        .write_stdin("2\n");
    cmd.assert().success().stdout(predicate::eq("b\n"));
}

#[test]
fn cli_pick_zero_exits_two_as_cancelled() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["a","b"]"#)
        .write_stdin("0\n");
    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty());
}

#[test]
fn cli_pick_empty_line_exits_two_as_cancelled() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["a","b"]"#)
        .write_stdin("\n");
    cmd.assert().failure().code(2);
}

#[test]
fn cli_pick_word_cancel_exits_two() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["a","b"]"#)
        .write_stdin("cancel\n");
    cmd.assert().failure().code(2);
}

#[test]
fn cli_invalid_pick_is_not_a_number() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["a","b"]"#)
        .write_stdin("x\n");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("is not a number"));
}

#[test]
fn cli_out_of_range_pick_is_named() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["a","b"]"#)
        .write_stdin("99\n");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("out of range"));
}

#[test]
fn cli_multiple_returns_comma_separated() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["a","b","c"]"#)
        .arg("--multiple")
        .write_stdin("1,3\n");
    cmd.assert().success().stdout(predicate::eq("a,c\n"));
}

#[test]
fn cli_empty_options_rejected() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("Pick")
        .arg("--options")
        .arg("[]");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("options is empty"));
}

#[test]
fn cli_single_option_rejected_as_nothing_to_ask() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["only"]"#);
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("only one option"));
}

#[test]
fn cli_missing_message_rejected() {
    let mut cmd = present();
    cmd.arg("ask").arg("--options").arg(r#"["a","b"]"#);
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("needs --message"));
}

#[test]
fn cli_missing_options_rejected() {
    let mut cmd = present();
    cmd.arg("ask").arg("--message").arg("Pick");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("needs --options"));
}

#[test]
fn cli_bad_options_json_rejected() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("Pick")
        .arg("--options")
        .arg("not json");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not a json array"));
}

#[test]
fn cli_empty_message_rejected() {
    let mut cmd = present();
    cmd.arg("ask")
        .arg("--message")
        .arg("")
        .arg("--options")
        .arg(r#"["a","b"]"#);
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("message is empty"));
}
