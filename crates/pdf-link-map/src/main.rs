use clap::{Parser, ValueEnum};
use lopdf::{Document, Object, Stream, dictionary};
use pdf_link_map::{audit_pdf, default_output_path, write_html};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{fs, io};

#[derive(Parser, Debug)]
#[command(
    name = "pdf-link-map",
    version,
    about = "Audit PDF links without opening them",
    long_about = "Inventory internal and external PDF link annotations, validate destinations, compare an optional heading manifest, and write a clickable standalone HTML map. External URLs are never requested and the PDF is never modified."
)]
struct Args {
    /// PDF file to inspect
    #[arg(
        value_name = "PDF",
        required_unless_present = "demo",
        conflicts_with = "demo"
    )]
    input: Option<PathBuf>,
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
    /// Audit a bundled sample PDF in a temporary directory and print the report path
    #[arg(long, conflicts_with_all = ["input", "output", "manifest"])]
    demo: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum FailOn {
    Never,
    Broken,
    Any,
}

fn main() -> ExitCode {
    let args = Args::parse();
    if args.demo {
        return run_demo(&args);
    }
    let input = args
        .input
        .as_deref()
        .expect("clap requires PDF without --demo");
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| default_output_path(input));
    if output.exists() && fs::canonicalize(&output).ok() == fs::canonicalize(input).ok() {
        eprintln!("pdf-link-map: report path must not overwrite the input PDF");
        return ExitCode::from(2);
    }
    finish_audit(
        input,
        args.manifest.as_deref(),
        &output,
        args.json,
        args.fail_on,
    )
}

fn run_demo(args: &Args) -> ExitCode {
    let directory = std::env::temp_dir().join(format!(
        "pdf-link-map-demo-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos())
    ));
    let input = directory.join("operations-handbook.pdf");
    let manifest = directory.join("headings.json");
    let output = directory.join("operations-handbook.link-map.html");
    if let Err(error) = write_demo_fixture(&input, &manifest) {
        eprintln!("pdf-link-map: could not create demo sample: {error}");
        return ExitCode::from(2);
    }
    eprintln!("Demo input: {}", input.display());
    let status = finish_audit(&input, Some(&manifest), &output, args.json, args.fail_on);
    eprintln!("Demo report: {}", output.display());
    status
}

fn finish_audit(
    input: &Path,
    manifest: Option<&Path>,
    output: &Path,
    json: bool,
    fail_on: FailOn,
) -> ExitCode {
    let report = match audit_pdf(input, manifest) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("pdf-link-map: {e}");
            return ExitCode::from(2);
        }
    };
    if let Err(e) = write_html(&report, output) {
        eprintln!("pdf-link-map: {e}");
        return ExitCode::from(2);
    }
    if json {
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
            input.display(),
            report.summary.total_links,
            report.summary.broken,
            report.summary.warnings
        );
        eprintln!("HTML map: {}", output.display());
    }
    let failed = match fail_on {
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

fn name(value: &str) -> Object {
    Object::Name(value.as_bytes().to_vec())
}

fn text(value: &str) -> Object {
    Object::string_literal(value)
}

fn link(destination: Object) -> Object {
    dictionary! {
        "Type" => "Annot", "Subtype" => "Link",
        "Rect" => vec![0.into(), 0.into(), 100.into(), 20.into()],
        "Dest" => destination
    }
    .into()
}

fn external_link(target: &str) -> Object {
    dictionary! {
        "Type" => "Annot", "Subtype" => "Link",
        "Rect" => vec![0.into(), 20.into(), 100.into(), 40.into()],
        "A" => dictionary! { "S" => "URI", "URI" => Object::string_literal(target) }
    }
    .into()
}

fn write_demo_fixture(pdf_path: &Path, manifest_path: &Path) -> io::Result<()> {
    fs::create_dir_all(pdf_path.parent().expect("demo fixture parent"))?;
    let mut document = Document::with_version("1.7");
    let pages_id = document.new_object_id();
    let first_page = document.new_object_id();
    let second_page = document.new_object_id();
    let first_content = document.add_object(Stream::new(dictionary! {}, Vec::new()));
    let second_content = document.add_object(Stream::new(dictionary! {}, Vec::new()));
    let install_destination = Object::Array(vec![second_page.into(), name("Fit")]);
    document.objects.insert(
        first_page,
        dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Resources" => dictionary! {}, "Contents" => first_content,
            "Annots" => vec![
                link(text("install")),
                link(text("missing-anchor")),
                external_link("https://example.test/docs")
            ]
        }
        .into(),
    );
    document.objects.insert(
        second_page,
        dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Resources" => dictionary! {}, "Contents" => second_content
        }
        .into(),
    );
    document.objects.insert(
        pages_id,
        dictionary! {
            "Type" => "Pages", "Kids" => vec![first_page.into(), second_page.into()], "Count" => 2
        }
        .into(),
    );
    let names = vec![text("install"), install_destination];
    let catalog = document.add_object(dictionary! {
        "Type" => "Catalog", "Pages" => pages_id,
        "Names" => dictionary! { "Dests" => dictionary! { "Names" => names } }
    });
    document.trailer.set("Root", catalog);
    document.compress();
    document.save(pdf_path)?;
    fs::write(
        manifest_path,
        r#"[{"title":"Installation","anchor":"install","page":2},{"title":"Lost chapter","anchor":"lost"}]"#,
    )?;
    Ok(())
}
