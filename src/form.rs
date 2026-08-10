use std::process::Command;

use crate::error::{PresentError, Result};
use crate::protocol::ShelfModule;

pub struct ParamForm {
    pub program: String,
    pub fields: Vec<ParamField>,
}

pub struct ParamField {
    pub label: String,
    pub default: String,
    pub value: String,
}

impl ParamForm {
    pub fn from_module(module: &ShelfModule) -> Result<Self> {
        let cli = module.entry.cli.as_deref().unwrap_or("").trim();
        if cli.is_empty() {
            return Err(PresentError::Usage(format!(
                "no entry point for {}. run sheol check {}",
                module.name, module.id
            )));
        }
        let tokens = tokenize_cli(cli);
        if tokens.is_empty() {
            return Err(PresentError::Usage(format!(
                "no entry point for {}. run sheol check {}",
                module.name, module.id
            )));
        }
        Ok(form_from_tokens(&tokens))
    }

    pub fn from_manifest(manifest: &crate::protocol::ModuleManifest) -> Result<Self> {
        let cli = manifest.entry.cli.as_deref().unwrap_or("").trim();
        if cli.is_empty() {
            return Err(PresentError::Usage(format!(
                "no entry point for {}. run sheol check {}",
                manifest.name, manifest.name
            )));
        }
        let tokens = tokenize_cli(cli);
        if tokens.is_empty() {
            return Err(PresentError::Usage(format!(
                "no entry point for {}. run sheol check {}",
                manifest.name, manifest.name
            )));
        }
        Ok(form_from_tokens(&tokens))
    }

    pub fn command_string(&self) -> String {
        let mut parts = vec![self.program.clone()];
        for field in &self.fields {
            parts.push(field.value.clone());
        }
        parts.join(" ")
    }

    pub fn run(&self) -> Result<()> {
        let mut cmd = Command::new(&self.program);
        for field in &self.fields {
            cmd.arg(&field.value);
        }
        let status = cmd
            .status()
            .map_err(|e| PresentError::Bad(format!(
                "could not run {}: {e}",
                self.command_string()
            )))?;
        let code = status.code().unwrap_or(1);
        if code != 0 {
            std::process::exit(code);
        }
        Ok(())
    }
}

pub fn fill_fields(form: &mut ParamForm) -> Result<()> {
    use std::io::{BufRead, Write};

    if form.fields.is_empty() {
        let mut stderr = std::io::stderr();
        let _ = writeln!(stderr, "entry:  {}", form.program);
        let _ = writeln!(stderr, "no arguments to fill. press enter to run, or empty to cancel");
        let stdin = std::io::stdin();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            return Err(PresentError::Bad("could not read your input".into()));
        }
        if line.trim().is_empty() {
            return Err(PresentError::Cancelled);
        }
        return Ok(());
    }

    for field in &mut form.fields {
        let mut stderr = std::io::stderr();
        let _ = write!(stderr, "  {} [{}]: ", field.label, field.default);
        let _ = stderr.flush();
        let stdin = std::io::stdin();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            return Err(PresentError::Bad("could not read your input".into()));
        }
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            field.value = trimmed.to_string();
        }
    }
    Ok(())
}

fn tokenize_cli(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';
    for c in s.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn form_from_tokens(tokens: &[String]) -> ParamForm {
    let program = tokens[0].clone();
    let fields: Vec<ParamField> = tokens[1..]
        .iter()
        .enumerate()
        .map(|(idx, tok)| ParamField {
            label: format!("arg {}", idx + 1),
            default: tok.clone(),
            value: tok.clone(),
        })
        .collect();
    ParamForm { program, fields }
}
