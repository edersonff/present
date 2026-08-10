use crate::protocol::AskResponse;
use std::io::Write;

#[derive(Debug)]
pub enum PresentError {
    Usage(String),
    Bad(String),
    Cancelled,
}

impl PresentError {
    pub fn code(&self) -> &'static str {
        match self {
            PresentError::Usage(_) => "usage",
            PresentError::Bad(_) => "bad-input",
            PresentError::Cancelled => "cancelled",
        }
    }

    pub fn message(&self) -> String {
        match self {
            PresentError::Usage(m) | PresentError::Bad(m) => m.clone(),
            PresentError::Cancelled => "cancelled".into(),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            PresentError::Usage(_) | PresentError::Bad(_) => 1,
            PresentError::Cancelled => 2,
        }
    }

    pub fn print_prose(&self) {
        let mut err = std::io::stderr();
        let _ = writeln!(err, "{}", self.message());
    }

    pub fn print_json(&self) {
        match self {
            PresentError::Cancelled => {
                crate::protocol::write_json(&AskResponse::cancelled());
            }
            PresentError::Usage(_) | PresentError::Bad(_) => {
                let mut out = std::io::stdout();
                let _ = serde_json::to_writer(
                    &mut out,
                    &serde_json::json!({
                        "error": self.message(),
                        "code": self.code(),
                    }),
                );
                let _ = out.write_all(b"\n");
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, PresentError>;
