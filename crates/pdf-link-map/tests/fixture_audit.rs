use lopdf::{Document, Object, Stream, dictionary};
use pdf_link_map::{LinkKind, LinkStatus, audit_pdf};
use std::fs;
use std::process::Command;

fn name(value: &str) -> Object {
    Object::Name(value.as_bytes().to_vec())
}
fn text(value: &str) -> Object {
    Object::string_literal(value)
}
fn link(dest: Object) -> Object {
    dictionary! { "Type" => "Annot", "Subtype" => "Link", "Rect" => vec![0.into(), 0.into(), 100.into(), 20.into()], "Dest" => dest }.into()
}
fn uri(value: &str) -> Object {
    dictionary! { "Type" => "Annot", "Subtype" => "Link", "Rect" => vec![0.into(), 20.into(), 100.into(), 40.into()], "A" => dictionary! { "S" => "URI", "URI" => Object::string_literal(value) } }.into()
}

fn fixture(path: &std::path::Path) {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let page_one = doc.new_object_id();
    let page_two = doc.new_object_id();
    let content_one = doc.add_object(Stream::new(dictionary! {}, Vec::new()));
    let content_two = doc.add_object(Stream::new(dictionary! {}, Vec::new()));
    let destination = Object::Array(vec![page_two.into(), name("Fit")]);
    let missing = text("missing-anchor");
    doc.objects.insert(
        page_one,
        dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Resources" => dictionary! {}, "Contents" => content_one,
            "Annots" => vec![link(text("install")), link(missing), uri("https://example.test/docs")]
        }
        .into(),
    );
    doc.objects.insert(
        page_two,
        dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Resources" => dictionary! {}, "Contents" => content_two
        }
        .into(),
    );
    doc.objects.insert(pages_id, dictionary! { "Type" => "Pages", "Kids" => vec![page_one.into(), page_two.into()], "Count" => 2 }.into());
    let names = vec![
        text("install"),
        destination.clone(),
        text("duplicate"),
        destination.clone(),
        text("duplicate"),
        destination,
    ];
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id, "Names" => dictionary! { "Dests" => dictionary! { "Names" => names } } });
    doc.trailer.set("Root", catalog);
    doc.compress();
    doc.save(path).unwrap();
}

fn fixture_without_links(path: &std::path::Path) {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let page = doc.new_object_id();
    let content = doc.add_object(Stream::new(dictionary! {}, Vec::new()));
    doc.objects.insert(
        page,
        dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Resources" => dictionary! {}, "Contents" => content
        }
        .into(),
    );
    doc.objects.insert(
        pages_id,
        dictionary! { "Type" => "Pages", "Kids" => vec![page.into()], "Count" => 1 }.into(),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog);
    doc.save(path).unwrap();
}

#[test]
fn catches_every_deliberately_broken_fixture_link() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("converted.pdf");
    fixture(&pdf);
    let manifest = dir.path().join("headings.json");
    fs::write(&manifest, r#"[{"title":"Install","anchor":"install","page":2},{"title":"Lost chapter","anchor":"lost"}]"#).unwrap();
    let report = audit_pdf(&pdf, Some(&manifest)).unwrap();
    assert_eq!(report.summary.total_links, 3);
    assert_eq!(
        report
            .links
            .iter()
            .filter(|link| link.status == LinkStatus::Broken)
            .count(),
        1
    );
    assert_eq!(
        report
            .links
            .iter()
            .filter(|link| link.kind == LinkKind::External)
            .count(),
        1
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "duplicate_destination")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "missing_manifest_anchor")
    );
    assert_eq!(report.links[0].target_page, Some(2));
}

#[test]
fn documented_json_ci_command_writes_report_and_fails_policy() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("handbook.pdf");
    fixture(&pdf);
    let html = dir.path().join("audit/link-map.html");
    let result = Command::new(env!("CARGO_BIN_EXE_pdf-link-map"))
        .arg(&pdf)
        .args(["--output"])
        .arg(&html)
        .args(["--json", "--fail-on", "broken"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["summary"]["total_links"], 3);
    let report = fs::read_to_string(html).unwrap();
    assert!(report.contains("Annotation map"));
    assert!(report.contains("missing-anchor"));
}

#[test]
fn malformed_input_exits_two_without_a_report() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("bad.pdf");
    fs::write(&input, b"not a pdf").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_pdf-link-map"))
        .arg(&input)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("Could not parse"));
}

#[test]
fn no_annotation_pdf_has_an_actionable_empty_state() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("flat.pdf");
    fixture_without_links(&pdf);
    let report = audit_pdf(&pdf, None).unwrap();
    assert_eq!(report.summary.total_links, 0);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "no_links")
    );
}

#[test]
fn refuses_to_overwrite_the_source_pdf() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("signed.pdf");
    fixture(&pdf);
    let before = fs::read(&pdf).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_pdf-link-map"))
        .arg(&pdf)
        .args(["--output"])
        .arg(&pdf)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert_eq!(fs::read(&pdf).unwrap(), before);
}

#[test]
fn demo_command_creates_a_real_sample_report() {
    let result = Command::new(env!("CARGO_BIN_EXE_pdf-link-map"))
        .args(["--demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["summary"]["total_links"], 3);
    assert_eq!(report["summary"]["external_links"], 1);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let report_path = stderr
        .lines()
        .find_map(|line| line.strip_prefix("Demo report: "))
        .expect("demo report path");
    assert!(std::path::Path::new(report_path).is_file());
}
