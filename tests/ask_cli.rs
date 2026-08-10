use assert_cmd::Command;
use predicates::prelude::*;

fn present() -> Command {
    Command::cargo_bin("present").unwrap()
}

#[test]
fn cli_pick_by_number_prints_choice_to_stdout() {
    let mut cmd = present();
    cmd.arg("--ask")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["a","b","c"]"#)
        .write_stdin("2\n");
    cmd.assert().success().stdout(predicate::eq("b\n"));
}

#[test]
fn cli_pick_zero_exits_two_as_cancelled() {
    let mut cmd = present();
    cmd.arg("--ask")
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
    cmd.arg("--ask")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["a","b"]"#)
        .write_stdin("\n");
    cmd.assert().failure().code(2);
}

#[test]
fn cli_pick_word_cancel_exits_two() {
    let mut cmd = present();
    cmd.arg("--ask")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["a","b"]"#)
        .write_stdin("cancel\n");
    cmd.assert().failure().code(2);
}

#[test]
fn cli_invalid_pick_is_not_a_number() {
    let mut cmd = present();
    cmd.arg("--ask")
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
    cmd.arg("--ask")
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
fn cli_empty_options_rejected() {
    let mut cmd = present();
    cmd.arg("--ask")
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
    cmd.arg("--ask")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["only"]"#);
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("only one option"));
}

#[test]
fn cli_ask_without_options_is_rejected() {
    let mut cmd = present();
    cmd.arg("--ask").arg("Pick");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("an ask needs options"));
}

#[test]
fn cli_bad_options_json_rejected() {
    let mut cmd = present();
    cmd.arg("--ask")
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
    cmd.arg("--ask")
        .arg("")
        .arg("--options")
        .arg(r#"["a","b"]"#);
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("message is empty"));
}

#[test]
fn cli_no_args_no_tty_errors_with_interactive_only_message() {
    let mut cmd = present();
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("interactive only"));
}

#[test]
fn cli_auto_pick_plain_mode_prints_choice_without_json() {
    let mut cmd = present();
    cmd.env("PRESENT_AUTO_PICK", "1")
        .arg("--ask")
        .arg("Pick")
        .arg("--options")
        .arg(r#"["first","second"]"#);
    cmd.assert().success().stdout(predicate::eq("first\n"));
}
