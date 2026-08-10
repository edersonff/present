use clap::{Parser, Subcommand};

mod ask;
mod error;
mod progress;
mod protocol;
mod tui;

use error::{PresentError, Result};

#[derive(Parser)]
#[command(
    name = "present",
    about = "surfaces a decision to the human and returns the pick",
    version,
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// ask the human to pick from a list
    Ask {
        /// the message shown above the options
        #[arg(long)]
        message: Option<String>,

        /// options as a json array, e.g. '["a","b","c"]'
        #[arg(long)]
        options: Option<String>,

        /// allow more than one pick
        #[arg(long, default_value_t = false)]
        multiple: bool,

        /// force an interactive picker. only works in a tty
        #[arg(long, default_value_t = false)]
        interactive: bool,

        /// read an ask request from stdin, write the response to stdout
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// read progress updates as json lines from stdin, render a bar to stderr
    Progress {
        /// read {current,total,label} json lines from stdin
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let use_json = matches!(cli.command, Command::Ask { json: true, .. })
        || matches!(cli.command, Command::Progress { json: true, .. });

    let result: Result<()> = match cli.command {
        Command::Ask { json: true, .. } => ask::run_json(),
        Command::Ask {
            message,
            options,
            multiple,
            interactive,
            json: false,
        } => ask_from_flags(message, options, multiple, interactive),
        Command::Progress { json: true } => progress::run_json(),
        Command::Progress { json: false } => Err(PresentError::Usage(
            "progress needs --json. pipe {current,total,label} lines on stdin".into(),
        )),
    };

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

fn ask_from_flags(
    message: Option<String>,
    options: Option<String>,
    multiple: bool,
    interactive: bool,
) -> Result<()> {
    let message = message.ok_or_else(|| {
        PresentError::Usage("present ask needs --message, or use --json for stdin".into())
    })?;
    let options_raw = options.ok_or_else(|| {
        PresentError::Usage(
            "present ask needs --options as a json array, e.g. '[\"a\",\"b\"]'".into(),
        )
    })?;
    let options: Vec<String> = serde_json::from_str(options_raw.as_str()).map_err(|err| {
        PresentError::Usage(format!(
            "--options is not a json array, line {} looks wrong",
            err.line().max(1)
        ))
    })?;
    ask::run_cli(&message, &options, multiple, interactive)
}
