use clap::{Parser, ValueEnum};
use pdf_link_map::{audit_pdf, default_output_path, write_html};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "pdf-link-map",
    version,
    about = "Audit PDF links without opening them",
    long_about = "Inventory internal and external PDF link annotations, validate destinations, compare an optional heading manifest, and write a clickable standalone HTML map. External URLs are never requested and the PDF is never modified."
)]
struct Args {
    /// PDF file to inspect
    #[arg(value_name = "PDF")]
    input: PathBuf,
    /// Where to write the standalone HTML report [default: <PDF>.link-map.html]
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Optional JSON heading manifest to compare with named destinations
    #[arg(short, long, value_name = "FILE")]
    manifest: Option<PathBuf>,
    /// Print the complete audit report as JSON to stdout
    #[arg(long)]
    json: bool,
    /// Exit 1 when the selected class of findings exists
    #[arg(long, value_enum, default_value_t = FailOn::Never)]
    fail_on: FailOn,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum FailOn {
    Never,
    Broken,
    Any,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let output = args
        .output
        .unwrap_or_else(|| default_output_path(&args.input));
    if output.exists()
        && std::fs::canonicalize(&output).ok() == std::fs::canonicalize(&args.input).ok()
    {
        eprintln!("pdf-link-map: report path must not overwrite the input PDF");
        return ExitCode::from(2);
    }
    let report = match audit_pdf(&args.input, args.manifest.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("pdf-link-map: {e}");
            return ExitCode::from(2);
        }
    };
    if let Err(e) = write_html(&report, &output) {
        eprintln!("pdf-link-map: {e}");
        return ExitCode::from(2);
    }
    if args.json {
        match serde_json::to_string_pretty(&report) {
            Ok(v) => println!("{v}"),
            Err(e) => {
                eprintln!("pdf-link-map: could not serialize results: {e}");
                return ExitCode::from(2);
            }
        }
    } else {
        eprintln!(
            "Audited {}: {} links, {} broken, {} warnings.",
            args.input.display(),
            report.summary.total_links,
            report.summary.broken,
            report.summary.warnings
        );
        eprintln!("HTML map: {}", output.display());
    }
    let failed = match args.fail_on {
        FailOn::Never => false,
        FailOn::Broken => report.summary.broken > 0,
        FailOn::Any => report.summary.broken > 0 || report.summary.warnings > 0,
    };
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
