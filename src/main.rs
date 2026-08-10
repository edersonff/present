use clap::{Parser, Subcommand};

mod ask;
mod error;
mod form;
mod progress;
mod protocol;
mod shelf;
mod tui;

use error::{PresentError, Result};

#[derive(Parser)]
#[command(
    name = "present",
    about = "pick a module or surface a decision. one binary, two faces.",
    version,
    long_about = None
)]
struct Cli {
    /// ask the human to pick from a list. the message text.
    #[arg(long)]
    ask: Option<String>,

    /// options as a json array, e.g. '["a","b","c"]'. needs --ask.
    #[arg(long)]
    options: Option<String>,

    /// jump to a module's param form by name or owner/repo
    #[arg(long)]
    module: Option<String>,

    /// answer as json on stdout, for a program reading this
    #[arg(long, default_value_t = false, global = true)]
    json: bool,

    /// force the interactive ratatui picker. needs a tty
    #[arg(long, default_value_t = false)]
    interactive: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// read progress updates as json lines from stdin, render a bar to stderr
    Progress,
}

fn main() {
    let cli = Cli::parse();

    let (result, use_json) = dispatch(&cli);

    match result {
        Ok(()) => {}
        Err(err) => {
            if use_json {
                err.print_json();
            } else {
                err.print_prose();
            }
            std::process::exit(err.exit_code());
        }
    }
}

fn dispatch(cli: &Cli) -> (Result<()>, bool) {
    if let Some(Command::Progress) = cli.command {
        return (progress::run(cli.json), cli.json);
    }

    if let Some(message) = &cli.ask {
        return (run_ask(cli, message), cli.json);
    }

    if let Some(name) = &cli.module {
        return (run_module(name), cli.json);
    }

    if cli.json {
        return (ask::run_from_stdin(), true);
    }

    (run_browse(), false)
}

fn run_ask(cli: &Cli, message: &str) -> Result<()> {
    let options = cli.options.as_deref().ok_or_else(|| {
        PresentError::Usage(
            "an ask needs options. --options '[\"a\",\"b\"]'".into(),
        )
    })?;
    ask::run_from_flags(message, options, cli.json, cli.interactive)
}

fn run_module(name: &str) -> Result<()> {
    let shelf = shelf::load()?;
    let module = shelf::find_by_name(&shelf, name).ok_or_else(|| {
        PresentError::Usage(format!(
            "no module named {name:?} on the shelf. run sheol search to see what is there"
        ))
    })?;
    run_module_form(module)
}

fn run_browse() -> Result<()> {
    if !tui::is_tty() {
        return Err(PresentError::Usage(
            "interactive only. pipe a question with --json or run me in a terminal.".into(),
        ));
    }

    if let Ok(manifest) = read_local_manifest() {
        return run_local_form(&manifest);
    }

    let shelf = shelf::load()?;
    let picked = tui::run_shelf_browser(&shelf.modules)?;
    match picked {
        Some(id) => {
            let module = shelf
                .modules
                .iter()
                .find(|m| m.id == id)
                .ok_or_else(|| PresentError::Bad(format!("picked {id} but could not find it")))?;
            run_module_form(module)
        }
        None => Err(PresentError::Cancelled),
    }
}

fn print_header(name: &str, description: &str, input: &str, output: &str) {
    use std::io::Write;
    let mut stderr = std::io::stderr();
    let _ = writeln!(stderr, "{name}");
    let _ = writeln!(stderr, "{description}");
    if !input.is_empty() {
        let _ = writeln!(stderr, "input:  {input}");
    }
    if !output.is_empty() {
        let _ = writeln!(stderr, "output: {output}");
    }
    let _ = writeln!(stderr);
}

fn run_module_form(module: &protocol::ShelfModule) -> Result<()> {
    print_header(&module.name, &module.description, &module.input, &module.output);
    let mut param_form = form::ParamForm::from_module(module)?;
    form::fill_fields(&mut param_form)?;
    eprintln!("running: {}", param_form.command_string());
    param_form.run()
}

fn run_local_form(manifest: &protocol::ModuleManifest) -> Result<()> {
    print_header(&manifest.name, &manifest.description, &manifest.input, &manifest.output);
    let mut param_form = form::ParamForm::from_manifest(manifest)?;
    form::fill_fields(&mut param_form)?;
    eprintln!("running: {}", param_form.command_string());
    param_form.run()
}

fn read_local_manifest() -> std::io::Result<protocol::ModuleManifest> {
    let text = std::fs::read_to_string("sheol.json")?;
    protocol::ModuleManifest::from_json_str(&text).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })
}
