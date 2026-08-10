use std::io::{BufRead, Write};

use crate::error::Result;
use crate::protocol::{write_json, ProgressUpdate};

pub fn run_json() -> Result<()> {
    let stdin = std::io::stdin();
    let mut last: Option<ProgressUpdate> = None;
    let mut bar = String::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let update = match ProgressUpdate::from_json_str(trimmed) {
            Ok(u) => u,
            Err(_) => {
                let _ = writeln!(
                    std::io::stderr(),
                    "present progress: a line was not valid json, skipped"
                );
                continue;
            }
        };
        render_bar(&update, &mut bar);
        last = Some(update);
    }

    if let Some(final_update) = last {
        render_final(&final_update);
    }
    write_json(&serde_json::json!({ "done": true }));
    Ok(())
}

const BAR_WIDTH: usize = 24;

fn render_bar(update: &ProgressUpdate, bar: &mut String) {
    let mut err = std::io::stderr();
    bar.clear();
    let filled = if update.total == 0 {
        0
    } else {
        ((update.current as f64 / update.total as f64) * BAR_WIDTH as f64).round() as usize
    };
    let filled = filled.min(BAR_WIDTH);
    let empty = BAR_WIDTH - filled;
    let pct = if update.total == 0 {
        0
    } else {
        ((update.current as f64 / update.total as f64) * 100.0).round() as u64
    };
    let label = if update.label.is_empty() {
        "working"
    } else {
        update.label.as_str()
    };
    *bar = format!(
        "\r{label} [{filled}{empty}] {pct}% ({current}/{total})\x1B[K",
        filled = "#".repeat(filled),
        empty = "-".repeat(empty),
        current = update.current,
        total = update.total,
    );
    let _ = err.write_all(bar.as_bytes());
    let _ = err.flush();
}

fn render_final(update: &ProgressUpdate) {
    let mut err = std::io::stderr();
    let _ = writeln!(
        err,
        "\ndone: {current}/{total} ({label})",
        current = update.current,
        total = update.total,
        label = if update.label.is_empty() {
            "completed"
        } else {
            update.label.as_str()
        }
    );
}
