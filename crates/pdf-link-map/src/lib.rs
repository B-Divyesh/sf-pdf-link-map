//! PDF annotation auditing as a small, typed library.
//!
//! ```no_run
//! use pdf_link_map::{audit_pdf, write_html};
//! use std::path::Path;
//! let report = audit_pdf(Path::new("handbook.pdf"), None)?;
//! write_html(&report, Path::new("handbook.link-map.html"))?;
//! # Ok::<(), pdf_link_map::AuditError>(())
//! ```

use lopdf::{Dictionary, Document, Object, ObjectId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub schema_version: u8,
    pub input: String,
    pub pages: usize,
    pub summary: Summary,
    pub links: Vec<LinkRecord>,
    pub destinations: Vec<DestinationRecord>,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Summary {
    pub total_links: usize,
    pub internal_links: usize,
    pub external_links: usize,
    pub valid_internal: usize,
    pub broken: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkRecord {
    pub id: String,
    pub source_page: u32,
    pub kind: LinkKind,
    pub target: String,
    pub target_page: Option<u32>,
    pub status: LinkStatus,
    pub note: Option<String>,
    pub rect: Option<[f64; 4]>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    Internal,
    External,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LinkStatus {
    Valid,
    Broken,
    External,
    Warning,
}

#[derive(Debug, Clone, Serialize)]
pub struct DestinationRecord {
    pub name: String,
    pub page: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub code: String,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManifestHeading {
    pub title: String,
    #[serde(default)]
    pub anchor: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug)]
pub enum AuditError {
    Io(String),
    Pdf(String),
    Manifest(String),
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(v) => write!(f, "{v}"),
            Self::Pdf(v) => write!(f, "{v}"),
            Self::Manifest(v) => write!(f, "{v}"),
        }
    }
}
impl std::error::Error for AuditError {}

pub fn audit_pdf(path: &Path, manifest_path: Option<&Path>) -> Result<AuditReport, AuditError> {
    if !path.exists() {
        return Err(AuditError::Io(format!("PDF not found: {}", path.display())));
    }
    let doc = Document::load(path)
        .map_err(|e| AuditError::Pdf(format!("Could not parse {}: {e}", path.display())))?;
    if doc.is_encrypted() {
        return Err(AuditError::Pdf(
            "Encrypted PDFs are not supported; decrypt a copy before auditing.".into(),
        ));
    }
    let pages = doc.get_pages();
    let page_by_id: HashMap<ObjectId, u32> = pages.iter().map(|(n, id)| (*id, *n)).collect();
    let mut findings = Vec::new();
    if pages.is_empty() {
        findings.push(finding(
            "empty_pdf",
            Severity::Warning,
            "The PDF has no pages.",
        ));
    }

    let named_pairs = collect_named_destinations(&doc);
    let mut seen = HashSet::new();
    let mut duplicates = HashSet::new();
    for (name, _) in &named_pairs {
        if !seen.insert(name.clone()) {
            duplicates.insert(name.clone());
        }
    }
    for name in &duplicates {
        findings.push(finding(
            "duplicate_destination",
            Severity::Error,
            format!("Named destination “{name}” is defined more than once."),
        ));
    }

    let mut named = BTreeMap::new();
    let mut destinations = Vec::new();
    for (name, object) in named_pairs {
        let page = destination_page(&doc, &object, &page_by_id);
        let valid = page.is_some();
        if !valid {
            findings.push(finding(
                "invalid_destination",
                Severity::Error,
                format!("Named destination “{name}” does not resolve to a page."),
            ));
        }
        named.entry(name.clone()).or_insert(object);
        destinations.push(DestinationRecord { name, page, valid });
    }
    destinations.sort_by(|a, b| a.name.cmp(&b.name));

    let mut links = Vec::new();
    for (page_number, page_id) in &pages {
        let page_dict = match doc.get_object(*page_id).and_then(Object::as_dict) {
            Ok(v) => v,
            Err(e) => {
                findings.push(finding(
                    "malformed_page",
                    Severity::Warning,
                    format!("Page {page_number} could not be inspected: {e}"),
                ));
                continue;
            }
        };
        let Some(annots_obj) = get_optional(page_dict, b"Annots") else {
            continue;
        };
        let Ok(annots_obj) = deref(&doc, annots_obj) else {
            findings.push(finding(
                "malformed_annotations",
                Severity::Warning,
                format!("Page {page_number} has an unreadable annotations array."),
            ));
            continue;
        };
        let Ok(annots) = annots_obj.as_array() else {
            findings.push(finding(
                "malformed_annotations",
                Severity::Warning,
                format!("Page {page_number} annotations are not an array."),
            ));
            continue;
        };
        for (index, annotation) in annots.iter().enumerate() {
            let id = format!("p{page_number}-l{}", index + 1);
            let Ok(annotation) = deref(&doc, annotation) else {
                links.push(broken_link(
                    id,
                    *page_number,
                    "Unreadable annotation object.",
                ));
                continue;
            };
            let Ok(dict) = annotation.as_dict() else {
                continue;
            };
            if dict.get(b"Subtype").ok().and_then(|v| v.as_name().ok()) != Some(b"Link") {
                continue;
            }
            let rect = parse_rect(dict.get(b"Rect").ok());
            let mut link = inspect_link(&doc, dict, *page_number, id, rect, &page_by_id, &named);
            if link.status == LinkStatus::Broken {
                findings.push(finding(
                    "broken_link",
                    Severity::Error,
                    format!(
                        "Link {} on page {}: {}",
                        link.id,
                        link.source_page,
                        link.note.as_deref().unwrap_or("invalid destination")
                    ),
                ));
            } else if link.status == LinkStatus::Warning {
                findings.push(finding(
                    "unsupported_link",
                    Severity::Warning,
                    format!(
                        "Link {} on page {}: {}",
                        link.id,
                        link.source_page,
                        link.note.as_deref().unwrap_or("unsupported action")
                    ),
                ));
            }
            link.rect = rect;
            links.push(link);
        }
    }

    if let Some(manifest_path) = manifest_path {
        let raw = fs::read_to_string(manifest_path).map_err(|e| {
            AuditError::Manifest(format!(
                "Could not read manifest {}: {e}",
                manifest_path.display()
            ))
        })?;
        let headings: Vec<ManifestHeading> = serde_json::from_str(&raw).map_err(|e| {
            AuditError::Manifest(format!("Manifest must be a JSON array of headings: {e}"))
        })?;
        validate_manifest(&headings, &doc, &named, &page_by_id, &mut findings);
    }

    if links.is_empty() {
        findings.push(finding("no_links", Severity::Warning, "No link annotations were found. If the source had navigation, the converter may have dropped it."));
    }
    let summary = Summary {
        total_links: links.len(),
        internal_links: links
            .iter()
            .filter(|x| x.kind == LinkKind::Internal)
            .count(),
        external_links: links
            .iter()
            .filter(|x| x.kind == LinkKind::External)
            .count(),
        valid_internal: links
            .iter()
            .filter(|x| x.kind == LinkKind::Internal && x.status == LinkStatus::Valid)
            .count(),
        broken: findings
            .iter()
            .filter(|x| x.severity == Severity::Error)
            .count(),
        warnings: findings
            .iter()
            .filter(|x| x.severity == Severity::Warning)
            .count(),
    };

    Ok(AuditReport {
        schema_version: 1,
        input: path.display().to_string(),
        pages: pages.len(),
        summary,
        links,
        destinations,
        findings,
    })
}

fn inspect_link(
    doc: &Document,
    dict: &Dictionary,
    source_page: u32,
    id: String,
    rect: Option<[f64; 4]>,
    page_by_id: &HashMap<ObjectId, u32>,
    named: &BTreeMap<String, Object>,
) -> LinkRecord {
    let destination = dict.get(b"Dest").ok().cloned();
    if let Some(dest) = destination {
        return internal_link(doc, dest, source_page, id, rect, page_by_id, named);
    }
    let Some(action) = get_optional(dict, b"A") else {
        return broken_link(id, source_page, "Link has neither /Dest nor /A.");
    };
    let Ok(action) = deref(doc, action) else {
        return broken_link(id, source_page, "Link action cannot be read.");
    };
    let Ok(action) = action.as_dict() else {
        return broken_link(id, source_page, "Link action is not a dictionary.");
    };
    let action_type = action
        .get(b"S")
        .ok()
        .and_then(|v| v.as_name().ok())
        .unwrap_or_default();
    match action_type {
        b"URI" => {
            let target = action.get(b"URI").ok().map(object_text).unwrap_or_default();
            if target.is_empty() {
                return broken_link(id, source_page, "URI action has no target.");
            }
            LinkRecord {
                id,
                source_page,
                kind: LinkKind::External,
                target,
                target_page: None,
                status: LinkStatus::External,
                note: Some("Recorded only; never requested.".into()),
                rect,
            }
        }
        b"GoTo" => match action.get(b"D") {
            Ok(dest) => internal_link(doc, dest.clone(), source_page, id, rect, page_by_id, named),
            Err(_) => broken_link(id, source_page, "GoTo action has no destination."),
        },
        _ => LinkRecord {
            id,
            source_page,
            kind: LinkKind::Unsupported,
            target: format!("/{}", String::from_utf8_lossy(action_type)),
            target_page: None,
            status: LinkStatus::Warning,
            note: Some("Action type is recorded but not validated in v1.".into()),
            rect,
        },
    }
}

fn internal_link(
    doc: &Document,
    dest: Object,
    source_page: u32,
    id: String,
    rect: Option<[f64; 4]>,
    page_by_id: &HashMap<ObjectId, u32>,
    named: &BTreeMap<String, Object>,
) -> LinkRecord {
    let (label, resolved) = match &dest {
        Object::Name(v) | Object::String(v, _) => {
            let name = String::from_utf8_lossy(v).into_owned();
            let resolved = named
                .get(&name)
                .and_then(|v| destination_page(doc, v, page_by_id));
            (name, resolved)
        }
        _ => (
            "explicit destination".into(),
            destination_page(doc, &dest, page_by_id),
        ),
    };
    let (status, note) = if resolved.is_some() {
        (LinkStatus::Valid, None)
    } else {
        (
            LinkStatus::Broken,
            Some(format!("Destination “{label}” does not resolve to a page.")),
        )
    };
    LinkRecord {
        id,
        source_page,
        kind: LinkKind::Internal,
        target: label,
        target_page: resolved,
        status,
        note,
        rect,
    }
}

fn destination_page(
    doc: &Document,
    object: &Object,
    page_by_id: &HashMap<ObjectId, u32>,
) -> Option<u32> {
    let object = deref(doc, object).ok()?;
    let object = if let Ok(dict) = object.as_dict() {
        dict.get(b"D")
            .ok()
            .and_then(|v| deref(doc, v).ok())
            .unwrap_or(object)
    } else {
        object
    };
    let array = object.as_array().ok()?;
    let first = array.first()?;
    match first {
        Object::Reference(id) => page_by_id.get(id).copied(),
        Object::Integer(index) if *index >= 0 => {
            Some(*index as u32 + 1).filter(|n| (*n as usize) <= page_by_id.len())
        }
        _ => None,
    }
}

fn collect_named_destinations(doc: &Document) -> Vec<(String, Object)> {
    let mut output = Vec::new();
    let Ok(catalog) = doc.catalog() else {
        return output;
    };
    if let Some(dests) = get_optional(catalog, b"Dests")
        .and_then(|v| deref(doc, v).ok())
        .and_then(|v| v.as_dict().ok())
    {
        for (name, dest) in dests.iter() {
            output.push((String::from_utf8_lossy(name).into_owned(), dest.clone()));
        }
    }
    if let Some(names) = get_optional(catalog, b"Names")
        .and_then(|v| deref(doc, v).ok())
        .and_then(|v| v.as_dict().ok())
        && let Some(tree) = get_optional(names, b"Dests")
    {
        collect_name_tree(doc, tree, &mut output, 0);
    }
    output
}

fn collect_name_tree(
    doc: &Document,
    object: &Object,
    output: &mut Vec<(String, Object)>,
    depth: u8,
) {
    if depth > 32 {
        return;
    }
    let Ok(object) = deref(doc, object) else {
        return;
    };
    let Ok(dict) = object.as_dict() else { return };
    if let Some(names) = get_optional(dict, b"Names")
        .and_then(|v| deref(doc, v).ok())
        .and_then(|v| v.as_array().ok())
    {
        for pair in names.chunks(2) {
            if pair.len() == 2 {
                output.push((object_text(&pair[0]), pair[1].clone()));
            }
        }
    }
    if let Some(kids) = get_optional(dict, b"Kids")
        .and_then(|v| deref(doc, v).ok())
        .and_then(|v| v.as_array().ok())
    {
        for kid in kids {
            collect_name_tree(doc, kid, output, depth + 1);
        }
    }
}

fn validate_manifest(
    headings: &[ManifestHeading],
    doc: &Document,
    named: &BTreeMap<String, Object>,
    page_by_id: &HashMap<ObjectId, u32>,
    findings: &mut Vec<Finding>,
) {
    for heading in headings {
        let Some(anchor) = &heading.anchor else {
            continue;
        };
        let Some(dest) = named.get(anchor) else {
            findings.push(finding(
                "missing_manifest_anchor",
                Severity::Error,
                format!(
                    "Manifest heading “{}” expects missing destination “{}”.",
                    heading.title, anchor
                ),
            ));
            continue;
        };
        if let Some(expected) = heading.page {
            let actual = destination_page(doc, dest, page_by_id);
            if actual != Some(expected) {
                findings.push(finding(
                    "manifest_page_mismatch",
                    Severity::Error,
                    format!(
                        "Manifest heading “{}” expects page {}, destination “{}” resolves to {}.",
                        heading.title,
                        expected,
                        anchor,
                        actual
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "no page".into())
                    ),
                ));
            }
        }
    }
}

fn deref<'a>(doc: &'a Document, object: &'a Object) -> Result<&'a Object, lopdf::Error> {
    match object {
        Object::Reference(id) => doc.get_object(*id),
        _ => Ok(object),
    }
}
fn get_optional<'a>(dict: &'a Dictionary, key: &[u8]) -> Option<&'a Object> {
    dict.get(key).ok()
}
fn object_text(object: &Object) -> String {
    match object {
        Object::String(v, _) | Object::Name(v) => String::from_utf8_lossy(v).into_owned(),
        _ => String::new(),
    }
}
fn parse_rect(value: Option<&Object>) -> Option<[f64; 4]> {
    let values = value?.as_array().ok()?;
    if values.len() != 4 {
        return None;
    }
    let mut out = [0.0; 4];
    for (i, value) in values.iter().enumerate() {
        out[i] = match value {
            Object::Integer(v) => *v as f64,
            Object::Real(v) => *v as f64,
            _ => return None,
        };
    }
    Some(out)
}
fn broken_link(id: String, source_page: u32, message: impl Into<String>) -> LinkRecord {
    LinkRecord {
        id,
        source_page,
        kind: LinkKind::Internal,
        target: "unresolved".into(),
        target_page: None,
        status: LinkStatus::Broken,
        note: Some(message.into()),
        rect: None,
    }
}
fn finding(code: &str, severity: Severity, message: impl Into<String>) -> Finding {
    Finding {
        code: code.into(),
        severity,
        message: message.into(),
    }
}

pub fn default_output_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or("report");
    input.with_file_name(format!("{stem}.link-map.html"))
}

pub fn write_html(report: &AuditReport, output: &Path) -> Result<(), AuditError> {
    let mut rows = String::new();
    for link in &report.links {
        let status = match link.status {
            LinkStatus::Valid => "✓ Valid",
            LinkStatus::Broken => "✕ Broken",
            LinkStatus::External => "↗ External",
            LinkStatus::Warning => "! Review",
        };
        let target = if link.kind == LinkKind::External && is_safe_external(&link.target) {
            format!(
                "<a href=\"{}\" rel=\"noreferrer\">{}</a>",
                esc(&link.target),
                esc(&link.target)
            )
        } else if let Some(page) = link.target_page {
            let pdf_href = if Path::new(&report.input).is_absolute() {
                report.input.clone()
            } else {
                format!("./{}", report.input)
            };
            format!(
                "<a href=\"{}#page={}\">page {}</a>",
                esc(&pdf_href),
                page,
                page
            )
        } else {
            esc(&link.target)
        };
        let _ = write!(
            rows,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td><span class=\"status {:?}\">{}</span></td><td>{}</td></tr>",
            link.source_page,
            esc(&link.id),
            target,
            link.status,
            status,
            esc(link.note.as_deref().unwrap_or("—"))
        );
    }
    if rows.is_empty() {
        rows.push_str("<tr><td colspan=\"5\" class=\"empty\">No link annotations found.</td></tr>");
    }
    let mut issues = String::new();
    for item in &report.findings {
        let _ = write!(
            issues,
            "<li><strong>{}</strong> {}</li>",
            esc(&item.code),
            esc(&item.message)
        );
    }
    if issues.is_empty() {
        issues.push_str("<li>✓ No findings.</li>");
    }
    let title = format!(
        "Link map — {}",
        Path::new(&report.input)
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or(&report.input)
    );
    let title_escaped = esc(&title);
    let html = format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title_escaped}</title><style>
:root{{--paper:#f4eedb;--raised:#fffdf5;--ink:#19232b;--muted:#51616b;--rule:#a8c1c5;--red:#8e2d25;--blue:#1f5872;--green:#2d684e}}*{{box-sizing:border-box}}body{{margin:0;background:var(--paper);color:var(--ink);font:16px/1.55 system-ui,sans-serif;background-image:repeating-linear-gradient(0deg,transparent 0 31px,rgba(70,120,130,.16) 31px 32px)}}main{{width:min(1120px,calc(100% - 32px));margin:auto;padding:48px 0 80px}}h1{{font-size:clamp(2rem,6vw,3.5rem);margin:.2em 0}}.summary{{display:flex;flex-wrap:wrap;gap:16px;margin:28px 0}}.metric{{background:var(--raised);border:1px solid var(--rule);padding:12px 18px;box-shadow:3px 3px 0 var(--ink)}}.metric strong{{display:block;font-size:1.6rem}}.table-wrap{{overflow:auto;background:var(--raised)}}.table-wrap:focus-visible{{outline:3px solid var(--blue);outline-offset:3px}}table{{border-collapse:collapse;width:100%;min-width:760px}}th,td{{padding:12px;text-align:left;border-bottom:1px solid var(--rule)}}th{{background:#e4e5d5}}a{{color:var(--blue);font-weight:650}}a:focus-visible{{outline:3px solid var(--blue);outline-offset:3px}}.status{{font-weight:750}}.Broken{{color:var(--red)}}.Valid{{color:var(--green)}}.empty{{padding:32px;text-align:center}}code{{background:var(--raised);padding:2px 5px}}footer{{margin-top:48px;color:var(--muted)}}@media print{{body{{background:white}}main{{width:100%;padding:0}}}}
</style></head><body><main><p>PDF LINK MAP / LOCAL AUDIT</p><h1>{}</h1><p><code>{}</code> · {} pages · generated locally</p><section class="summary" aria-label="Audit summary"><div class="metric"><strong>{}</strong>links</div><div class="metric"><strong>{}</strong>valid internal</div><div class="metric"><strong>{}</strong>external</div><div class="metric"><strong>{}</strong>broken</div></section><h2>Annotation map</h2><div class="table-wrap" tabindex="0" role="region" aria-label="Scrollable annotation map"><table><thead><tr><th>From page</th><th>ID</th><th>Destination</th><th>Status</th><th>Note</th></tr></thead><tbody>{rows}</tbody></table></div><h2>Findings</h2><ul>{issues}</ul><footer>External addresses are listed without being requested. PDF Link Map never modifies the source file.</footer></main></body></html>"#,
        esc(&title),
        esc(&report.input),
        report.pages,
        report.summary.total_links,
        report.summary.valid_internal,
        report.summary.external_links,
        report.summary.broken
    );
    if let Some(parent) = output.parent().filter(|v| !v.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|e| AuditError::Io(format!("Could not create {}: {e}", parent.display())))?;
    }
    fs::write(output, html)
        .map_err(|e| AuditError::Io(format!("Could not write report {}: {e}", output.display())))
}

fn is_safe_external(value: &str) -> bool {
    ["https://", "http://", "mailto:"]
        .iter()
        .any(|prefix| value.starts_with(prefix))
}
fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_report_name() {
        assert_eq!(
            default_output_path(Path::new("guide.pdf")),
            PathBuf::from("guide.link-map.html")
        );
    }
    #[test]
    fn unsafe_uri_is_not_clickable() {
        let mut report = AuditReport {
            schema_version: 1,
            input: "x.pdf".into(),
            pages: 1,
            summary: Summary::default(),
            links: vec![],
            destinations: vec![],
            findings: vec![],
        };
        report.links.push(LinkRecord {
            id: "x".into(),
            source_page: 1,
            kind: LinkKind::External,
            target: "javascript:alert(1)".into(),
            target_page: None,
            status: LinkStatus::External,
            note: None,
            rect: None,
        });
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.html");
        write_html(&report, &path).unwrap();
        let html = fs::read_to_string(path).unwrap();
        assert!(!html.contains("href=\"javascript:"));
    }
}
