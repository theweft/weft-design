// SPDX-FileCopyrightText: 2026 The weft authors
// SPDX-License-Identifier: MIT

//! Checks the design docs against `facts.toml`, the single source of truth for key values.
//!
//! - Every inline marker `<!--f:ID-->TEXT<!--/f-->` in a Markdown file must name a known fact,
//!   and TEXT must equal the fact's `text` (or its value).
//! - Every `[[fact.code]]` pattern must occur in the named file of the named repository.
//! - `FACTS.md` must match what `factcheck index` generates.
//! - External facts older than their `max_age_days` and unused facts are warnings.
//! - Requirements across repositories are consistent (see `requirements.rs`).
//!
//! Usage: `factcheck [check|index] [--root DIR] [--repos DIR] [--deny-warnings]`

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use toml::Value;

mod requirements;

const FACTS_FILE: &str = "facts.toml";
const INDEX_FILE: &str = "FACTS.md";
const OPEN: &str = "<!--f:";
const CLOSE: &str = "<!--/f-->";
const DEFAULT_MAX_AGE_DAYS: i64 = 180;
const KINDS: [&str; 5] = [
    "decision",
    "requirement",
    "assumption",
    "measurement",
    "external",
];
const STATUSES: [&str; 3] = ["draft", "decided", "superseded"];

struct CodeRef {
    repo: String,
    file: String,
    pattern: String,
}

struct Fact {
    id: String,
    value: Value,
    text: Option<String>,
    kind: String,
    status: String,
    note: String,
    source: Option<String>,
    checked: Option<String>,
    measured: Option<String>,
    max_age_days: i64,
    code: Vec<CodeRef>,
}

impl Fact {
    /// What a doc marker must contain.
    fn expected_text(&self) -> String {
        self.text.clone().unwrap_or_else(|| render(&self.value))
    }
}

#[derive(Default)]
pub(crate) struct Report {
    errors: Vec<String>,
    warnings: Vec<String>,
}

fn render(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(items) => items.iter().map(render).collect::<Vec<_>>().join(", "),
        other => other.to_string(),
    }
}

fn get_str(t: &toml::Table, key: &str) -> Option<String> {
    t.get(key).map(|v| match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    })
}

#[allow(clippy::too_many_lines)] // one linear validation pass; splitting it would not help reading
fn parse_facts(src: &str, report: &mut Report) -> Vec<Fact> {
    let table: toml::Table = match src.parse() {
        Ok(t) => t,
        Err(e) => {
            report.errors.push(format!("{FACTS_FILE}: {e}"));
            return Vec::new();
        }
    };
    let Some(Value::Array(entries)) = table.get("fact") else {
        report
            .errors
            .push(format!("{FACTS_FILE}: no [[fact]] entries"));
        return Vec::new();
    };
    let known = [
        "id",
        "value",
        "text",
        "kind",
        "status",
        "note",
        "source",
        "checked",
        "measured",
        "max_age_days",
        "code",
    ];
    let mut facts = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in entries {
        let Value::Table(t) = entry else {
            report
                .errors
                .push(format!("{FACTS_FILE}: [[fact]] must be a table"));
            continue;
        };
        let Some(id) = get_str(t, "id") else {
            report.errors.push(format!("{FACTS_FILE}: fact without id"));
            continue;
        };
        let err = |msg: &str| format!("{FACTS_FILE}: {id}: {msg}");
        for key in t.keys() {
            if !known.contains(&key.as_str()) {
                report.errors.push(err(&format!("unknown field `{key}`")));
            }
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c))
        {
            report.errors.push(err("id must match [a-z0-9._-]+"));
        }
        if !seen.insert(id.clone()) {
            report.errors.push(err("duplicate id"));
        }
        let Some(value) = t.get("value").cloned() else {
            report.errors.push(err("missing value"));
            continue;
        };
        let kind = get_str(t, "kind").unwrap_or_default();
        if !KINDS.contains(&kind.as_str()) {
            report
                .errors
                .push(err(&format!("kind must be one of {KINDS:?}")));
        }
        let status = get_str(t, "status").unwrap_or_else(|| "draft".into());
        if !STATUSES.contains(&status.as_str()) {
            report
                .errors
                .push(err(&format!("status must be one of {STATUSES:?}")));
        }
        let source = get_str(t, "source");
        let checked = get_str(t, "checked");
        let measured = get_str(t, "measured");
        if kind == "external" && (source.is_none() || checked.is_none()) {
            report
                .errors
                .push(err("external facts need `source` and `checked`"));
        }
        if kind == "measurement" && measured.is_none() {
            report.errors.push(err("measurements need `measured`"));
        }
        let max_age_days = match t.get("max_age_days") {
            Some(Value::Integer(n)) => *n,
            Some(_) => {
                report.errors.push(err("max_age_days must be an integer"));
                DEFAULT_MAX_AGE_DAYS
            }
            None => DEFAULT_MAX_AGE_DAYS,
        };
        let mut code = Vec::new();
        if let Some(Value::Array(refs)) = t.get("code") {
            for r in refs {
                let Value::Table(r) = r else { continue };
                match (
                    get_str(r, "repo"),
                    get_str(r, "file"),
                    get_str(r, "pattern"),
                ) {
                    (Some(repo), Some(file), Some(pattern)) => code.push(CodeRef {
                        repo,
                        file,
                        pattern,
                    }),
                    _ => report
                        .errors
                        .push(err("code entries need repo, file and pattern")),
                }
            }
        }
        facts.push(Fact {
            text: get_str(t, "text"),
            note: get_str(t, "note").unwrap_or_default(),
            id,
            value,
            kind,
            status,
            source,
            checked,
            measured,
            max_age_days,
            code,
        });
    }
    facts
}

/// All Markdown files under `root`, skipping build output and hidden directories.
fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "target" {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("md"))
                && name != INDEX_FILE
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Blanks out fenced code blocks and inline code spans (keeping line breaks), so examples of
/// markers in code are not checked.
fn mask_code(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut in_fence = false;
    for line in src.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            out.push_str(&blank(line));
            continue;
        }
        if in_fence {
            out.push_str(&blank(line));
            continue;
        }
        let mut in_span = false;
        for c in line.chars() {
            if c == '`' {
                in_span = !in_span;
                out.push(' ');
            } else if in_span && c != '\n' {
                out.push(' ');
            } else {
                out.push(c);
            }
        }
    }
    out
}

fn blank(line: &str) -> String {
    line.chars()
        .map(|c| if c == '\n' { c } else { ' ' })
        .collect()
}

/// Markers in one file: (line, id, text).
fn markers(src: &str, file: &str, report: &mut Report) -> Vec<(usize, String, String)> {
    let mut out = Vec::new();
    let mut rest = src;
    let mut offset = 0;
    while let Some(start) = rest.find(OPEN) {
        let line = src[..offset + start].lines().count().max(1);
        let after = &rest[start + OPEN.len()..];
        let Some(id_end) = after.find("-->") else {
            report
                .errors
                .push(format!("{file}:{line}: unterminated marker"));
            break;
        };
        let id = after[..id_end].trim().to_string();
        let body = &after[id_end + 3..];
        let Some(text_end) = body.find(CLOSE) else {
            report
                .errors
                .push(format!("{file}:{line}: marker `{id}` without {CLOSE}"));
            break;
        };
        let text = &body[..text_end];
        if text.contains(OPEN) {
            report
                .errors
                .push(format!("{file}:{line}: nested marker in `{id}`"));
        }
        out.push((line, id, text.to_string()));
        let consumed = start + OPEN.len() + id_end + 3 + text_end + CLOSE.len();
        offset += consumed;
        rest = &rest[consumed..];
    }
    out
}

/// Days since 1970-01-01 for a `YYYY-MM-DD` date (proleptic Gregorian).
fn days_from_date(date: &str) -> Option<i64> {
    let mut parts = date.get(..10)?.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

fn today_days() -> i64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    i64::try_from(secs / 86_400).unwrap_or(0)
}

fn index(facts: &[Fact]) -> String {
    let mut out = String::from(
        "<!-- Generated by tools/factcheck from facts.toml. Do not edit; run `just facts-index`. -->\n\n\
         # Facts\n\n\
         Key values, decisions and assumptions of weft. The source of truth is [facts.toml](facts.toml); \
         docs refer to entries with inline markers, and `just check` verifies them.\n",
    );
    let mut groups: BTreeMap<&str, Vec<&Fact>> = BTreeMap::new();
    for f in facts {
        let group = f.id.split('.').next().unwrap_or("");
        groups.entry(group).or_default().push(f);
    }
    for (group, items) in groups {
        let _ = write!(
            out,
            "\n## {group}\n\n| Id | Value | Kind | Status | Note |\n|---|---|---|---|---|\n"
        );
        for f in items {
            let mut note = f.note.clone();
            if let (Some(src), Some(date)) = (&f.source, &f.checked) {
                let _ = write!(note, " ([source]({src}), checked {date})");
            }
            if let Some(date) = &f.measured {
                let _ = write!(note, " (measured {date})");
            }
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} | {} |",
                f.id,
                f.expected_text().replace('|', "\\|"),
                f.kind,
                f.status,
                note.trim().replace('|', "\\|")
            );
        }
    }
    out
}

fn check(root: &Path, repos: &Path, facts: &[Fact], report: &mut Report) {
    let by_id: BTreeMap<&str, &Fact> = facts.iter().map(|f| (f.id.as_str(), f)).collect();
    let mut used: BTreeSet<&str> = BTreeSet::new();

    for path in markdown_files(root) {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let Ok(src) = std::fs::read_to_string(&path) else {
            report.errors.push(format!("{rel}: cannot read"));
            continue;
        };
        for (line, id, text) in markers(&mask_code(&src), &rel, report) {
            match by_id.get(id.as_str()) {
                None => report
                    .errors
                    .push(format!("{rel}:{line}: unknown fact `{id}`")),
                Some(f) => {
                    used.insert(f.id.as_str());
                    let expected = f.expected_text();
                    if text != expected {
                        report.errors.push(format!(
                            "{rel}:{line}: `{id}` says \"{text}\", facts.toml says \"{expected}\""
                        ));
                    }
                }
            }
        }
    }

    let reqs = requirements::collect(root, repos, report);
    let fact_ids: BTreeSet<&str> = by_id.keys().copied().collect();
    let req_facts = requirements::check(&reqs, &fact_ids, report);
    used.extend(
        facts
            .iter()
            .filter(|f| req_facts.contains(&f.id))
            .map(|f| f.id.as_str()),
    );

    for f in facts {
        for c in &f.code {
            used.insert(f.id.as_str());
            let pattern = c.pattern.replace("{value}", &render(&f.value));
            let repo_dir = repos.join(&c.repo);
            if !repo_dir.is_dir() {
                report.warnings.push(format!(
                    "{}: repository `{}` not found under {} (code check skipped)",
                    f.id,
                    c.repo,
                    repos.display()
                ));
                continue;
            }
            match std::fs::read_to_string(repo_dir.join(&c.file)) {
                Ok(src) if src.contains(&pattern) => {}
                Ok(_) => report.errors.push(format!(
                    "{}: `{}` not found in {}/{}",
                    f.id, pattern, c.repo, c.file
                )),
                Err(_) => report
                    .errors
                    .push(format!("{}: cannot read {}/{}", f.id, c.repo, c.file)),
            }
        }
        if !used.contains(f.id.as_str()) && f.status != "superseded" {
            report
                .warnings
                .push(format!("{}: not referenced by any doc or code", f.id));
        }
        if let Some(date) = &f.checked {
            match days_from_date(date) {
                Some(d) if today_days() - d > f.max_age_days => report.warnings.push(format!(
                    "{}: last checked {date}, older than {} days: re-check the source",
                    f.id, f.max_age_days
                )),
                Some(_) => {}
                None => report
                    .errors
                    .push(format!("{}: invalid date `{date}`", f.id)),
            }
        }
    }

    let expected_index = index(facts);
    match std::fs::read_to_string(root.join(INDEX_FILE)) {
        Ok(current) if current == expected_index => {}
        _ => report.errors.push(format!(
            "{INDEX_FILE} is out of date: run `just facts-index`"
        )),
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut command = String::from("check");
    let mut root = PathBuf::from(".");
    let mut repos: Option<PathBuf> = None;
    let mut deny_warnings = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "check" | "index" => command = arg,
            "--root" => root = args.next().map_or(root, PathBuf::from),
            "--repos" => repos = args.next().map(PathBuf::from),
            "--deny-warnings" => deny_warnings = true,
            other => {
                eprintln!("unknown argument `{other}`");
                return ExitCode::FAILURE;
            }
        }
    }
    let repos = repos.unwrap_or_else(|| root.join(".."));

    let mut report = Report::default();
    let src = match std::fs::read_to_string(root.join(FACTS_FILE)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cannot read {FACTS_FILE}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let facts = parse_facts(&src, &mut report);

    if command == "index" && report.errors.is_empty() {
        if let Err(e) = std::fs::write(root.join(INDEX_FILE), index(&facts)) {
            eprintln!("cannot write {INDEX_FILE}: {e}");
            return ExitCode::FAILURE;
        }
        println!("wrote {INDEX_FILE} ({} facts)", facts.len());
        return ExitCode::SUCCESS;
    }
    if command == "check" {
        check(&root, &repos, &facts, &mut report);
    }

    for w in &report.warnings {
        eprintln!("warning: {w}");
    }
    for e in &report.errors {
        eprintln!("error: {e}");
    }
    let failed = !report.errors.is_empty() || (deny_warnings && !report.warnings.is_empty());
    if failed {
        ExitCode::FAILURE
    } else {
        println!(
            "facts ok: {} facts, {} warnings",
            facts.len(),
            report.warnings.len()
        );
        ExitCode::SUCCESS
    }
}
