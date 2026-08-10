use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

use crate::error::{PresentError, Result};
use crate::protocol::{read_stdin_to_string, write_json, AskRequest, AskResponse};

pub fn run_json() -> Result<()> {
    let raw = read_stdin_to_string()
        .map_err(|e| PresentError::Bad(format!("could not read stdin: {e}")))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(PresentError::Bad(
            "no input on stdin, present --json needs an ask request there".into(),
        ));
    }
    let request = AskRequest::from_json_str(trimmed).map_err(|err| {
        let full = err.to_string();
        let reason = full.split(" at line ").next().unwrap_or(&full);
        PresentError::Bad(format!(
            "stdin is not a valid ask request: {reason}. line {}",
            err.line().max(1)
        ))
    })?;

    if std::env::var("PRESENT_AUTO_PICK").is_ok() {
        write_json(&AskResponse::selected(vec![request.options[0].clone()]));
        return Ok(());
    }

    let picked = pick_from_tty(&request)?;
    match picked {
        Some(items) => {
            write_json(&AskResponse::selected(items));
            Ok(())
        }
        None => {
            write_json(&AskResponse::cancelled());
            Ok(())
        }
    }
}

pub fn run_cli(message: &str, options: &[String], multiple: bool, interactive: bool) -> Result<()> {
    let request = validate(message, options, multiple)?;
    if interactive {
        if !crate::tui::is_tty() {
            return Err(PresentError::Usage(
                "present --interactive needs a terminal. drop the flag to use the plain prompt, or run it in a tty"
                    .into(),
            ));
        }
        let picked = crate::tui::run(&request)?;
        return match picked {
            Some(items) => {
                println!("{}", items.join(","));
                Ok(())
            }
            None => Err(PresentError::Cancelled),
        };
    }
    let picked = pick_from_stdin(&request)?;
    match picked {
        Some(items) => {
            println!("{}", items.join(","));
            Ok(())
        }
        None => Err(PresentError::Cancelled),
    }
}

fn validate(message: &str, options: &[String], multiple: bool) -> Result<AskRequest> {
    if message.trim().is_empty() {
        return Err(PresentError::Usage(
            "message is empty, present needs something to ask".into(),
        ));
    }
    match options.len() {
        0 => Err(PresentError::Usage(
            "options is empty, present needs at least two to ask".into(),
        )),
        1 => Err(PresentError::Usage(
            "only one option was given, nothing to ask. pass it through or add another".into(),
        )),
        _ => Ok(AskRequest {
            message: message.to_string(),
            options: options.to_vec(),
            multiple,
        }),
    }
}

fn pick_from_stdin(request: &AskRequest) -> Result<Option<Vec<String>>> {
    prompt_to_stderr(request);
    let stdin = std::io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_err() {
        return Err(PresentError::Bad(
            "could not read your pick from stdin".into(),
        ));
    }
    parse_pick(request, line.trim())
}

fn pick_from_tty(request: &AskRequest) -> Result<Option<Vec<String>>> {
    let mut tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .map_err(|e| PresentError::Bad(format!(
            "no terminal to ask in ({e}). present --json reads the request from stdin and the pick from /dev/tty. set PRESENT_AUTO_PICK=1 to auto-select the first option without a human"
        )))?;
    prompt_to_writer(request, &mut tty)
        .map_err(|e| PresentError::Bad(format!("could not write prompt to /dev/tty: {e}")))?;
    let mut reader = BufReader::new(&mut tty);
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return Err(PresentError::Bad(
            "could not read your pick from /dev/tty".into(),
        ));
    }
    parse_pick(request, line.trim())
}

fn prompt_to_stderr(request: &AskRequest) {
    let mut err = std::io::stderr();
    if let Err(e) = prompt_to_writer(request, &mut err) {
        let _ = writeln!(err, "could not write prompt: {e}");
    }
}

fn prompt_to_writer<W: Write>(request: &AskRequest, w: &mut W) -> std::io::Result<()> {
    writeln!(w, "{}", request.message)?;
    for (idx, opt) in request.options.iter().enumerate() {
        writeln!(w, "  ({}) {}", idx + 1, opt)?;
    }
    if request.multiple {
        writeln!(
            w,
            "pick one or more by number, comma-separated. 0 or empty cancels"
        )
    } else {
        writeln!(w, "pick one by number. 0 or empty cancels")
    }
}
fn parse_pick(request: &AskRequest, trimmed: &str) -> Result<Option<Vec<String>>> {
    if trimmed.is_empty() || trimmed == "0" || trimmed.eq_ignore_ascii_case("cancel") {
        return Ok(None);
    }
    if request.multiple {
        let mut picks: Vec<String> = Vec::new();
        for part in trimmed.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let n: usize = part.parse().map_err(|_| {
                PresentError::Bad(format!(
                    "{part:?} is not a number, enter a digit between 1 and {}",
                    request.options.len()
                ))
            })?;
            let Some(opt) = request.options.get(n.checked_sub(1).unwrap_or(usize::MAX)) else {
                return Err(PresentError::Bad(format!(
                    "{n} is out of range, pick between 1 and {}",
                    request.options.len()
                )));
            };
            picks.push(opt.clone());
        }
        if picks.is_empty() {
            return Ok(None);
        }
        Ok(Some(picks))
    } else {
        let n: usize = trimmed.parse().map_err(|_| {
            PresentError::Bad(format!(
                "{trimmed:?} is not a number, enter a digit between 1 and {}",
                request.options.len()
            ))
        })?;
        let Some(opt) = request.options.get(n.checked_sub(1).unwrap_or(usize::MAX)) else {
            return Err(PresentError::Bad(format!(
                "{n} is out of range, pick between 1 and {}",
                request.options.len()
            )));
        };
        Ok(Some(vec![opt.clone()]))
    }
}
