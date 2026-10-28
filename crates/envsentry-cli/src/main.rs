use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use envsentry_core::{
    ComparisonOptions, ComparisonReport, Diagnostic, Severity, compare_documents,
};
use envsentry_parsers::{parse_dotenv, parse_json, parse_toml};

#[derive(Debug, Parser)]
#[command(
    name = "envsentry",
    version,
    about = "Detect configuration drift without exposing secrets.",
    arg_required_else_help = true
)]
struct Cli {
    #[arg(long, global = true, value_enum, default_value = "text")]
    format: OutputFormat,

    #[arg(long, global = true, help = "Treat warnings as policy failures")]
    strict: bool,

    #[arg(
        long,
        global = true,
        help = "Enable redact-all policy (reports currently never include values)"
    )]
    redact_all: bool,

    #[arg(long, global = true, help = "Disable colored terminal output")]
    no_color: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Compare a target configuration against a reference configuration.
    Compare { reference: PathBuf, target: PathBuf },
    /// Check a target configuration against a schema, ignoring value differences.
    Check {
        #[arg(long)]
        schema: PathBuf,
        target: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("envsentry: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<u8, String> {
    let (reference_path, target_path, compare_values, success_message) = match &cli.command {
        Command::Compare { reference, target } => {
            (reference, target, true, "No configuration drift detected.")
        }
        Command::Check { schema, target } => (schema, target, false, "Configuration check passed."),
    };

    let reference = load_document(reference_path)?;
    let target = load_document(target_path)?;
    let report = compare_documents(
        &reference,
        &target,
        ComparisonOptions::new().compare_values(compare_values),
    );

    let use_color = !cli.no_color && cli.format == OutputFormat::Text && io::stdout().is_terminal();
    let stdout = io::stdout();
    let mut output = stdout.lock();

    match cli.format {
        OutputFormat::Text => render_text(&mut output, &report, success_message, use_color),
        OutputFormat::Json => render_json(&mut output, &report),
    }
    .map_err(|_| "failed to write report".to_owned())?;

    let failed = report.has_errors() || (cli.strict && report.has_warnings());
    let _redact_all = cli.redact_all;
    Ok(if failed { 1 } else { 0 })
}

fn load_document(path: &Path) -> Result<envsentry_core::ConfigDocument, String> {
    let input = std::fs::read_to_string(path)
        .map_err(|_| format!("cannot read input file '{}'", escaped_path(path)))?;
    let format = infer_format(path).ok_or_else(|| {
        format!(
            "unsupported file format for '{}'; expected .env, .json, or .toml",
            escaped_path(path)
        )
    })?;

    let document =
        match format {
            InputFormat::Dotenv => parse_dotenv(&input)
                .map_err(|error| format!("{} in '{}'", error, escaped_path(path)))?,
            InputFormat::Json => parse_json(&input)
                .map_err(|error| format!("{} in '{}'", error, escaped_path(path)))?,
            InputFormat::Toml => parse_toml(&input)
                .map_err(|error| format!("{} in '{}'", error, escaped_path(path)))?,
        };

    Ok(document.with_source_name(path.to_string_lossy().into_owned()))
}

#[derive(Clone, Copy)]
enum InputFormat {
    Dotenv,
    Json,
    Toml,
}

fn infer_format(path: &Path) -> Option<InputFormat> {
    let file_name = path.file_name()?.to_string_lossy();
    let lower_name = file_name.to_ascii_lowercase();

    if lower_name == ".env"
        || lower_name.starts_with(".env.")
        || lower_name.ends_with(".dotenv")
        || path.extension().is_none()
    {
        return Some(InputFormat::Dotenv);
    }

    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "json" => Some(InputFormat::Json),
        "toml" => Some(InputFormat::Toml),
        "env" => Some(InputFormat::Dotenv),
        _ => None,
    }
}

fn render_text(
    output: &mut impl Write,
    report: &ComparisonReport,
    success_message: &str,
    color: bool,
) -> io::Result<()> {
    if report.is_clean() {
        writeln!(output, "{success_message}")?;
        return Ok(());
    }

    for diagnostic in report.diagnostics() {
        render_diagnostic(output, diagnostic, color)?;
    }

    let errors = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.severity() == Severity::Error)
        .count();
    let warnings = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.severity() == Severity::Warning)
        .count();
    writeln!(output, "{errors} error(s), {warnings} warning(s)")
}

fn render_diagnostic(
    output: &mut impl Write,
    diagnostic: &Diagnostic,
    color: bool,
) -> io::Result<()> {
    let severity = match diagnostic.severity() {
        Severity::Info => "INFO",
        Severity::Warning => "WARNING",
        Severity::Error => "ERROR",
    };
    if color {
        let ansi = match diagnostic.severity() {
            Severity::Info => "\x1b[36m",
            Severity::Warning => "\x1b[33m",
            Severity::Error => "\x1b[31m",
        };
        write!(output, "{ansi}{severity}\x1b[0m {}", diagnostic.code())?;
    } else {
        write!(output, "{severity} {}", diagnostic.code())?;
    }

    if let Some(path) = diagnostic.path() {
        write!(output, ": {path}")?;
    }
    if let Some(source) = diagnostic.source_name() {
        let source = escape_controls(source);
        write!(output, " ({source}")?;
        if let Some(location) = diagnostic.location() {
            write!(output, ":{}:{}", location.line(), location.column())?;
        }
        write!(output, ")")?;
    }
    writeln!(output)
}

fn escaped_path(path: &Path) -> String {
    escape_controls(&path.to_string_lossy())
}

fn escape_controls(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                character.escape_default().to_string()
            } else {
                character.to_string()
            }
        })
        .collect()
}

fn render_json(output: &mut impl Write, report: &ComparisonReport) -> io::Result<()> {
    let diagnostics: Vec<_> = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            let mut item = serde_json::Map::new();
            item.insert(
                "code".into(),
                serde_json::Value::String(diagnostic.code().as_str().to_owned()),
            );
            item.insert(
                "severity".into(),
                serde_json::Value::String(
                    match diagnostic.severity() {
                        Severity::Info => "info",
                        Severity::Warning => "warning",
                        Severity::Error => "error",
                    }
                    .to_owned(),
                ),
            );
            item.insert(
                "path".into(),
                diagnostic
                    .path()
                    .map(|path| serde_json::Value::String(path.to_string()))
                    .unwrap_or(serde_json::Value::Null),
            );
            item.insert(
                "source".into(),
                diagnostic
                    .source_name()
                    .map(|source| serde_json::Value::String(source.to_owned()))
                    .unwrap_or(serde_json::Value::Null),
            );
            item.insert(
                "line".into(),
                diagnostic
                    .location()
                    .map(|location| serde_json::Value::from(location.line()))
                    .unwrap_or(serde_json::Value::Null),
            );
            item.insert(
                "column".into(),
                diagnostic
                    .location()
                    .map(|location| serde_json::Value::from(location.column()))
                    .unwrap_or(serde_json::Value::Null),
            );
            serde_json::Value::Object(item)
        })
        .collect();

    let document = serde_json::json!({
        "status": if report.is_clean() { "clean" } else { "findings" },
        "diagnostic_count": diagnostics.len(),
        "diagnostics": diagnostics,
    });
    serde_json::to_writer_pretty(&mut *output, &document).map_err(io::Error::other)?;
    writeln!(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_supported_formats_case_insensitively() {
        assert!(matches!(infer_format(Path::new(".env.production")), Some(InputFormat::Dotenv)));
        assert!(matches!(infer_format(Path::new("settings.JSON")), Some(InputFormat::Json)));
        assert!(matches!(infer_format(Path::new("settings.TOML")), Some(InputFormat::Toml)));
        assert!(infer_format(Path::new("settings.yaml")).is_none());
    }

    #[test]
    fn text_escaping_prevents_control_characters_in_source_names() {
        assert_eq!(escape_controls("config\n\u{1b}[31m"), "config\\n\\u{1b}[31m");
        assert!(!escape_controls("config\n\u{1b}[31m").contains('\n'));
        assert!(!escape_controls("config\n\u{1b}[31m").contains('\u{1b}'));
    }

    #[test]
    fn json_report_never_contains_values() {
        let secret = "CLI_SECRET_SENTINEL";
        let reference = parse_json(&format!(r#"{{"TOKEN":"{secret}"}}"#)).unwrap();
        let target = parse_json(r#"{"TOKEN":"different"}"#).unwrap();
        let report = compare_documents(&reference, &target, ComparisonOptions::default());
        let mut output = Vec::new();

        render_json(&mut output, &report).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(!output.contains(secret));
        assert!(!output.contains("different"));
        assert!(output.contains("VALUE_MISMATCH"));
    }
}
