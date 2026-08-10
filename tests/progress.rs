use assert_cmd::Command;
use predicates::prelude::*;

fn present() -> Command {
    Command::cargo_bin("present").unwrap()
}

#[test]
fn progress_happy_path_exits_zero_and_writes_done() {
    let mut cmd = present();
    cmd.arg("progress").arg("--json").write_stdin(
        "{\"current\":1,\"total\":3,\"label\":\"a\"}\n\
             {\"current\":2,\"total\":3,\"label\":\"b\"}\n\
             {\"current\":3,\"total\":3,\"label\":\"c\"}\n",
    );
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""done":true"#));
}

#[test]
fn progress_empty_stdin_still_exits_zero() {
    let mut cmd = present();
    cmd.arg("progress").arg("--json").write_stdin("");
    cmd.assert().success();
}

#[test]
fn progress_malformed_line_is_skipped_not_fatal() {
    let mut cmd = present();
    cmd.arg("progress").arg("--json").write_stdin(
        "not json\n\
             {\"current\":1,\"total\":2,\"label\":\"ok\"}\n",
    );
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("skipped"))
        .stdout(predicate::str::contains(r#""done":true"#));
}

#[test]
fn progress_without_json_flag_is_rejected() {
    let mut cmd = present();
    cmd.arg("progress");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("needs --json"));
}

#[test]
fn progress_total_zero_does_not_crash() {
    let mut cmd = present();
    cmd.arg("progress")
        .arg("--json")
        .write_stdin("{\"current\":0,\"total\":0,\"label\":\"nothing\"}\n");
    cmd.assert().success();
}

#[test]
fn progress_label_optional() {
    let mut cmd = present();
    cmd.arg("progress")
        .arg("--json")
        .write_stdin("{\"current\":1,\"total\":2}\n");
    cmd.assert().success();
}
