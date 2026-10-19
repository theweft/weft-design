// SPDX-FileCopyrightText: 2026 The weft authors
// SPDX-License-Identifier: MIT

//! Cross-repository requirements check.
//!
//! System requirements live in `requirements/*.sdoc` of this repository; component requirements
//! live in `requirements/*.sdoc` of sibling repositories and name their parents in `SATISFIES`.
//!
//! - Requirement ids are unique across all repositories.
//! - Every `SATISFIES` entry names a known requirement, and every `FACTS` entry a known fact.
//! - Every Active system requirement verified by Test is satisfied by a component requirement.
//!
//! Coverage of components by code and tests is checked inside each repository (its own tests).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::Report;

pub struct Requirement {
    uid: String,
    status: String,
    verification: String,
    satisfies: Vec<String>,
    facts: Vec<String>,
    /// `repo/requirements/file.sdoc:line`
    location: String,
    system: bool,
}

fn list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

fn parse(src: &str, location: &str, system: bool) -> Vec<Requirement> {
    let mut out = Vec::new();
    let mut current: Option<Requirement> = None;
    for (n, line) in src.lines().enumerate() {
        if line == "[REQUIREMENT]" {
            out.extend(current.take());
            current = Some(Requirement {
                uid: String::new(),
                status: String::new(),
                verification: String::new(),
                satisfies: Vec::new(),
                facts: Vec::new(),
                location: format!("{location}:{}", n + 1),
                system,
            });
        } else if line.starts_with('[') {
            // Any other node ends the current requirement.
            out.extend(current.take());
        } else if let Some(r) = current.as_mut() {
            if let Some(v) = line.strip_prefix("UID: ") {
                r.uid = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("STATUS: ") {
                r.status = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("VERIFICATION: ") {
                r.verification = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("SATISFIES: ") {
                r.satisfies = list(v);
            } else if let Some(v) = line.strip_prefix("FACTS: ") {
                r.facts = list(v);
            }
        }
    }
    out.extend(current);
    out
}

fn sdoc_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "sdoc"))
        .collect();
    out.sort();
    out
}

/// System requirements of `root`, plus component requirements of every sibling under `repos`.
pub fn collect(root: &Path, repos: &Path, report: &mut Report) -> Vec<Requirement> {
    let root_canon = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut dirs: Vec<(PathBuf, String, bool)> =
        vec![(root.join("requirements"), "requirements".to_string(), true)];
    let mut siblings: Vec<PathBuf> = std::fs::read_dir(repos)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    siblings.sort();
    for repo in siblings {
        if repo.canonicalize().ok().as_deref() == Some(root_canon.as_path()) {
            continue;
        }
        let name = repo
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        dirs.push((
            repo.join("requirements"),
            format!("{name}/requirements"),
            false,
        ));
    }
    let mut out = Vec::new();
    for (dir, label, system) in dirs {
        for path in sdoc_files(&dir) {
            let file = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            match std::fs::read_to_string(&path) {
                Ok(src) => out.extend(parse(&src, &format!("{label}/{file}"), system)),
                Err(_) => report.errors.push(format!("{label}/{file}: cannot read")),
            }
        }
    }
    out
}

/// Checks references and coverage; returns the fact ids that requirements refer to.
pub fn check(
    reqs: &[Requirement],
    fact_ids: &BTreeSet<&str>,
    report: &mut Report,
) -> BTreeSet<String> {
    let mut by_uid: BTreeMap<&str, &Requirement> = BTreeMap::new();
    for r in reqs {
        if r.uid.is_empty() {
            report
                .errors
                .push(format!("{}: requirement without UID", r.location));
        } else if let Some(first) = by_uid.insert(r.uid.as_str(), r) {
            report.errors.push(format!(
                "{}: duplicate requirement {} (also at {})",
                r.location, r.uid, first.location
            ));
        }
    }
    let mut satisfied: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut used_facts = BTreeSet::new();
    for r in reqs {
        for parent in &r.satisfies {
            if by_uid.contains_key(parent.as_str()) {
                if r.status != "Deleted" {
                    satisfied
                        .entry(parent.as_str())
                        .or_default()
                        .push(r.uid.as_str());
                }
            } else {
                report.errors.push(format!(
                    "{}: {} satisfies unknown requirement `{parent}`",
                    r.location, r.uid
                ));
            }
        }
        for fact in &r.facts {
            if fact_ids.contains(fact.as_str()) {
                used_facts.insert(fact.clone());
            } else {
                report.errors.push(format!(
                    "{}: {} refers to unknown fact `{fact}`",
                    r.location, r.uid
                ));
            }
        }
    }
    let mut system = 0;
    let mut components = 0;
    for r in reqs {
        if r.system {
            system += 1;
            if r.status == "Active"
                && r.verification == "Test"
                && !satisfied.contains_key(r.uid.as_str())
            {
                report.errors.push(format!(
                    "{}: {} is Active and verified by Test, but no component requirement satisfies it",
                    r.location, r.uid
                ));
            }
        } else {
            components += 1;
        }
    }
    if system == 0 {
        report
            .warnings
            .push("no system requirements found in requirements/".to_string());
    }
    println!("requirements: {system} system, {components} component");
    used_facts
}
