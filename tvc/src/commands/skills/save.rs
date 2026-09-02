//! Skills save command - writes the agent skills bundled in the binary to a
//! skills directory where AI coding tools discover them.

use crate::outcome::Outcome;
use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use tracing::instrument;

pub(crate) const LONG_ABOUT: &str = r#"
Write the agent skills bundled with this tvc release to a skills directory.

Skills follow the open Agent Skills format (SKILL.md), which is read by Claude
Code, Codex, Cursor, and other tools that scan a skills directory. The bundled
skills document the tvc CLI at exactly the version of this binary, so re-run
this command after upgrading tvc.

By default the skills are written to ./.claude/skills (the project-level
directory). Use --global for ~/.claude/skills, or --dir for any other skills
root. Files you have modified locally are never overwritten unless --force is
passed; unmodified and absent files are always safe to (re)write."#;

/// The skill directories bundled into the binary, as
/// `(path relative to the skills root, content)`.
///
/// Evals are deliberately not bundled: they are test fixtures for the skill's
/// own CI, not something an agent consumes. The `embedded_manifest_matches_
/// skills_dir` test keeps this list in sync with `tvc/skills/` on disk.
const EMBEDDED_SKILL_FILES: [(&str, &str); 4] = [
    (
        "tvc-deployments/SKILL.md",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/skills/tvc-deployments/SKILL.md"
        )),
    ),
    (
        "tvc-deployments/references/deploy-lifecycle.md",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/skills/tvc-deployments/references/deploy-lifecycle.md"
        )),
    ),
    (
        "tvc-deployments/references/config-files.md",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/skills/tvc-deployments/references/config-files.md"
        )),
    ),
    (
        "tvc-deployments/references/error-reference.md",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/skills/tvc-deployments/references/error-reference.md"
        )),
    ),
];

/// The names of the bundled skills (the top-level directories under the
/// skills root).
const SKILL_NAMES: [&str; 1] = ["tvc-deployments"];

/// Save the bundled agent skills to a skills directory.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Skills root directory to write into (each skill becomes a
    /// subdirectory of it). Defaults to ./.claude/skills.
    #[arg(
        long,
        value_name = "DIR",
        env = "TVC_SKILLS_DIR",
        conflicts_with = "global"
    )]
    pub dir: Option<PathBuf>,

    /// Write to the user-level skills directory (~/.claude/skills) instead of
    /// the project-level default (./.claude/skills).
    #[arg(long)]
    pub global: bool,

    /// Overwrite skill files whose content differs from this binary's bundled
    /// version.
    #[arg(long)]
    pub force: bool,
}

/// Run the skills save command.
#[instrument(skip_all)]
pub fn run(args: Args) -> Result<Outcome> {
    let dest = match (args.dir, args.global) {
        (Some(dir), _) => dir,
        (None, true) => {
            let home = std::env::var("HOME").context("HOME environment variable not set")?;
            PathBuf::from(home).join(".claude").join("skills")
        }
        (None, false) => PathBuf::from(".claude").join("skills"),
    };

    // Plan first, write second: a conflict must abort before anything is
    // written, so a refused run never leaves a half-updated skill behind.
    let mut to_write = Vec::new();
    let mut unchanged = 0usize;
    let mut conflicts = Vec::new();

    for (relative_path, content) in EMBEDDED_SKILL_FILES {
        let path = dest.join(relative_path);
        if path.exists() {
            let existing = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read existing file: {}", path.display()))?;
            if existing == content {
                unchanged += 1;
                continue;
            }
            if !args.force {
                conflicts.push(relative_path);
                continue;
            }
        }
        to_write.push((path, content));
    }

    if !conflicts.is_empty() {
        bail!(
            "refusing to overwrite locally modified skill files: {}. \
             Pass --force to replace them with this binary's bundled version",
            conflicts.join(", ")
        );
    }

    for (path, content) in &to_write {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }
        std::fs::write(path, content)
            .with_context(|| format!("failed to write file: {}", path.display()))?;
    }

    Ok(Outcome::SkillsSaved(SkillsSaved {
        command: "skills save",
        path: dest.display().to_string(),
        skills: SKILL_NAMES.to_vec(),
        files_written: to_write.len(),
        files_unchanged: unchanged,
    }))
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsSaved {
    command: &'static str,
    path: String,
    skills: Vec<&'static str>,
    files_written: usize,
    files_unchanged: usize,
}

impl Display for SkillsSaved {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Saved agent skills to {} ({} written, {} unchanged):",
            self.path, self.files_written, self.files_unchanged
        )?;
        for skill in &self.skills {
            writeln!(f, "  - {skill}")?;
        }
        write!(
            f,
            r#"
Tools that support the Agent Skills format (Claude Code, Codex, Cursor, ...)
discover skills in this directory. The skills describe the tvc CLI at this
binary's version; re-run `tvc skills save` after upgrading tvc."#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Walk `tvc/skills/` on disk and collect every file that must be
    /// embedded, as paths relative to the skills root. Eval fixtures are the
    /// deliberate exception (see `EMBEDDED_SKILL_FILES`).
    fn skill_files_on_disk(dir: &Path, root: &Path, files: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("skills directory must be readable") {
            let path = entry.expect("directory entry must be readable").path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name == "evals") {
                    continue;
                }
                skill_files_on_disk(&path, root, files);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("walked file must live under the skills root");
                files.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }

    #[test]
    fn embedded_manifest_matches_skills_dir() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("skills");
        let mut on_disk = Vec::new();
        skill_files_on_disk(&root, &root, &mut on_disk);
        on_disk.sort();

        let mut embedded: Vec<String> = EMBEDDED_SKILL_FILES
            .iter()
            .map(|(path, _)| (*path).to_string())
            .collect();
        embedded.sort();

        assert_eq!(
            embedded, on_disk,
            "EMBEDDED_SKILL_FILES is out of sync with tvc/skills/ — register \
             new skill files in the manifest (evals stay excluded)"
        );
    }

    #[test]
    fn every_skill_has_a_skill_md() {
        for skill in SKILL_NAMES {
            assert!(
                EMBEDDED_SKILL_FILES
                    .iter()
                    .any(|(path, _)| *path == format!("{skill}/SKILL.md")),
                "skill `{skill}` has no embedded SKILL.md"
            );
        }
    }
}
