use std::process::Command;

use crate::error::{PresentError, Result};
use crate::protocol::Shelf;

pub fn load() -> Result<Shelf> {
    let output = Command::new("sheol")
        .arg("search")
        .arg("--json")
        .output()
        .map_err(|_| PresentError::Usage(
            "sheol is not on PATH. install it: cargo build --release && install -m755 target/release/sheol ~/.local/bin/sheol".into(),
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let trimmed = stderr.trim();
        return Err(PresentError::Bad(format!(
            "sheol search failed: {}",
            if trimmed.is_empty() { "unknown error" } else { trimmed }
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err(PresentError::Bad(
            "sheol search returned nothing. the shelf may be empty or unreachable".into(),
        ));
    }

    Shelf::from_json_str(trimmed).map_err(|err| {
        PresentError::Bad(format!(
            "sheol search returned json present could not parse: {err}"
        ))
    })
}

pub fn find_by_name<'a>(shelf: &'a Shelf, query: &str) -> Option<&'a crate::protocol::ShelfModule> {
    let lower = query.to_lowercase();
    shelf
        .modules
        .iter()
        .find(|m| m.name == query || m.id == query || m.name == lower)
        .or_else(|| {
            shelf.modules.iter().find(|m| {
                m.name.contains(&lower) || m.id.contains(&lower)
            })
        })
}
