//! Builds a Markdown changelog from Conventional Commits, for the release
//! workflow's GitHub Release body.
//!
//! Usage: generate_changelog --repo owner/name --to <ref> [--from <ref>]
//! Prints Markdown to stdout. --from may be omitted/empty to cover full
//! history.

use std::process::{Command, exit};

const RECORD_SEP: char = '\u{1e}';
const UNIT_SEP: char = '\u{1f}';

// Section headers double as Gitmoji (https://gitmoji.dev) badges
const BREAKING_CHANGES: &str = "\u{1f4a5} Breaking Changes";
const FEATURES: &str = "\u{2728} Features";
const BUG_FIXES: &str = "\u{1f41b} Bug Fixes";
const PERFORMANCE: &str = "\u{26a1} Performance";
const CODE_REFACTORING: &str = "\u{267b}\u{fe0f} Code Refactoring";
const DOCUMENTATION: &str = "\u{1f4dd} Documentation";
const STYLING: &str = "\u{1f484} Styling";
const TESTS: &str = "\u{2705} Tests";
const BUILD_SYSTEM: &str = "\u{1f4e6} Build System";
const CI_CD: &str = "\u{1f477} CI/CD";
const CHORES: &str = "\u{1f527} Chores";
const REVERTS: &str = "\u{23ea}\u{fe0f} Reverts";
const OTHER_CHANGES: &str = "\u{1f539} Other Changes";

const SECTION_ORDER: &[&str] = &[
    BREAKING_CHANGES,
    FEATURES,
    BUG_FIXES,
    PERFORMANCE,
    CODE_REFACTORING,
    DOCUMENTATION,
    STYLING,
    TESTS,
    BUILD_SYSTEM,
    CI_CD,
    CHORES,
    REVERTS,
    OTHER_CHANGES,
];

fn type_section(commit_type: &str) -> &'static str {
    match commit_type {
        "feat" => FEATURES,
        "fix" => BUG_FIXES,
        "perf" => PERFORMANCE,
        "refactor" => CODE_REFACTORING,
        "docs" => DOCUMENTATION,
        "style" => STYLING,
        "test" => TESTS,
        "build" => BUILD_SYSTEM,
        "ci" => CI_CD,
        "chore" => CHORES,
        "revert" => REVERTS,
        _ => OTHER_CHANGES,
    }
}

struct RawCommit {
    sha: String,
    short_sha: String,
    subject: String,
    body: String,
}

struct Entry {
    section: &'static str,
    label: String,
    link: String,
    description: String,
}

fn is_trailer_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.starts_with("signed-off-by:")
        || lower.starts_with("co-authored-by:")
        || lower.starts_with("reviewed-by:")
}

fn breaking_footer_text(line: &str) -> Option<&str> {
    let lower = line.to_ascii_lowercase();
    for prefix in ["breaking change:", "breaking-change:"] {
        if lower.starts_with(prefix) {
            return Some(line[prefix.len()..].trim());
        }
    }
    None
}

fn clean_body(body: &str) -> String {
    let mut kept = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if is_trailer_line(trimmed) || breaking_footer_text(trimmed).is_some() {
            continue;
        }
        kept.push(line);
    }
    kept.join("\n").trim().to_string()
}

fn extract_breaking_note(body: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some(first) = breaking_footer_text(trimmed) else {
            continue;
        };
        let mut collected: Vec<&str> = Vec::new();
        if !first.is_empty() {
            collected.push(first);
        }
        for cont in &lines[i + 1..] {
            let stripped = cont.trim();
            if stripped.is_empty() || is_trailer_line(stripped) || breaking_footer_text(stripped).is_some() {
                break;
            }
            collected.push(stripped);
        }
        let note = collected.join(" ").trim().to_string();
        if !note.is_empty() {
            return Some(note);
        }
        return None;
    }
    None
}

fn parse_subject(subject: &str) -> (Option<String>, Option<String>, bool, String) {
    let Some(colon_idx) = subject.find(':') else {
        return (None, None, false, subject.to_string());
    };
    let (prefix, rest) = subject.split_at(colon_idx);
    let desc = rest[1..].trim().to_string();
    if desc.is_empty() {
        return (None, None, false, subject.to_string());
    }

    let mut prefix = prefix;
    let mut breaking = false;
    if let Some(stripped) = prefix.strip_suffix('!') {
        breaking = true;
        prefix = stripped;
    }

    let (type_part, scope) = if prefix.ends_with(')') {
        match prefix.find('(') {
            Some(open) => (&prefix[..open], Some(prefix[open + 1..prefix.len() - 1].to_string())),
            None => (prefix, None),
        }
    } else {
        (prefix, None)
    };

    let is_valid_type = !type_part.is_empty() && type_part.chars().all(|c| c.is_ascii_alphabetic());
    if !is_valid_type {
        return (None, None, false, subject.to_string());
    }

    (Some(type_part.to_ascii_lowercase()), scope, breaking, desc)
}

fn load_commits(repo_dir: &str, from_ref: &str, to_ref: &str) -> Vec<RawCommit> {
    let range_spec = if from_ref.is_empty() {
        to_ref.to_string()
    } else {
        format!("{from_ref}..{to_ref}")
    };
    let fmt = format!("%H{UNIT_SEP}%h{UNIT_SEP}%s{UNIT_SEP}%b{RECORD_SEP}");

    let output = Command::new("git")
        .args(["-C", repo_dir, "log", &range_spec, &format!("--pretty=format:{fmt}"), "--no-merges"])
        .output()
        .expect("failed to run git log");
    if !output.status.success() {
        eprintln!("git log failed: {}", String::from_utf8_lossy(&output.stderr));
        exit(1);
    }
    let raw = String::from_utf8_lossy(&output.stdout);

    let mut commits = Vec::new();
    for record in raw.split(RECORD_SEP) {
        let record = record.trim_start_matches('\n');
        if record.is_empty() {
            continue;
        }
        let mut parts = record.splitn(4, UNIT_SEP);
        let (Some(sha), Some(short_sha), Some(subject), Some(body)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        commits.push(RawCommit {
            sha: sha.to_string(),
            short_sha: short_sha.to_string(),
            subject: subject.trim().to_string(),
            body: body.trim_matches('\n').to_string(),
        });
    }
    commits
}

fn parse_commit(commit: &RawCommit, repo: &str) -> Entry {
    let (commit_type, scope, mut breaking, desc) = parse_subject(&commit.subject);
    let breaking_note = extract_breaking_note(&commit.body);
    if breaking_note.is_some() {
        breaking = true;
    }

    let section = if breaking {
        BREAKING_CHANGES
    } else {
        type_section(commit_type.as_deref().unwrap_or(""))
    };

    let label = match scope {
        Some(scope) => format!("**{scope}**: {desc}"),
        None => desc,
    };
    let link = format!("[`{}`]({})", commit.short_sha, format_args!("https://github.com/{repo}/commit/{}", commit.sha));

    let description = if breaking {
        breaking_note.unwrap_or_default()
    } else {
        clean_body(&commit.body)
    };

    Entry { section, label, link, description }
}

fn render(entries: &[Entry]) -> String {
    if entries.is_empty() {
        return "No user-facing changes in this release.\n".to_string();
    }

    let mut out = String::new();
    for &section in SECTION_ORDER {
        let items: Vec<&Entry> = entries.iter().filter(|e| e.section == section).collect();
        if items.is_empty() {
            continue;
        }
        out.push_str(&format!("## {section}\n\n"));
        for item in items {
            out.push_str(&format!("- {} ({})\n", item.label, item.link));
            if !item.description.is_empty() {
                for para in item.description.split("\n\n") {
                    let para = para.trim();
                    if para.is_empty() {
                        continue;
                    }
                    let indented: Vec<String> = para.lines().map(|l| format!("  {l}")).collect();
                    out.push_str(&format!("\n{}\n\n", indented.join("\n")));
                }
            }
        }
        out.push('\n');
    }
    out.trim().to_string() + "\n"
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut repo = String::new();
    let mut to_ref = "HEAD".to_string();
    let mut from_ref = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--repo" => {
                i += 1;
                repo = args.get(i).expect("--repo requires a value").clone();
            }
            "--to" => {
                i += 1;
                to_ref = args.get(i).expect("--to requires a value").clone();
            }
            "--from" => {
                i += 1;
                from_ref = args.get(i).cloned().unwrap_or_default();
            }
            other => {
                eprintln!("unrecognized argument: {other}");
                exit(1);
            }
        }
        i += 1;
    }

    if repo.is_empty() {
        eprintln!("--repo is required");
        exit(1);
    }

    let commits = load_commits(".", &from_ref, &to_ref);
    let entries: Vec<Entry> = commits.iter().map(|c| parse_commit(c, &repo)).collect();
    print!("{}", render(&entries));
}
