//! Dateien aus Git lesen: alle versionierten (CI) oder nur die vorgemerkten (Pre-Commit-Haken).
use std::process::Command;

/// Einzige Stelle, an der das Werkzeug ein Programm startet: `git` mit festen Argumenten.
#[allow(clippy::disallowed_methods, reason = "SICHERHEIT-AUSNAHMEN A-013")]
fn git(args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|_| "git lässt sich nicht starten".to_string())?;
    if !out.status.success() {
        return Err(format!(
            "git {} ist fehlgeschlagen",
            args.first().unwrap_or(&"")
        ));
    }
    Ok(out.stdout)
}

/// Pfadliste aus `git … -z` (durch NUL getrennt). Nicht als UTF-8 lesbare Namen werden gemeldet.
pub fn split_names(raw: &[u8]) -> Result<Vec<String>, String> {
    raw.split(|c| *c == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8(s.to_vec()).map_err(|_| "Dateiname ist kein UTF-8".to_string()))
        .collect()
}

pub fn tracked() -> Result<Vec<String>, String> {
    split_names(&git(&["ls-files", "-z"])?)
}

pub fn staged() -> Result<Vec<String>, String> {
    split_names(&git(&[
        "diff",
        "--cached",
        "--name-only",
        "-z",
        "--diff-filter=ACMR",
    ])?)
}

/// Inhalt so, wie er committet würde (aus dem Index, nicht aus dem Arbeitsordner).
pub fn staged_content(path: &str) -> Result<Vec<u8>, String> {
    git(&["show", &format!(":{path}")])
}

/// Inhalt im Arbeitsordner; `None`, wenn die Datei dort gelöscht ist.
pub fn worktree_content(path: &str) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

/// Woher die Prüfungen lesen: Arbeitsordner (lokal), Index (Pre-Commit-Haken) oder der Commit `HEAD` (CI). In der
/// CI zählt nur `HEAD`: Was Tests oder Build-Skripte im Arbeitsordner ändern, sieht die Prüfung so nicht
/// (Review Prüflücken H-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Worktree,
    Staged,
    Head,
}

impl Source {
    pub fn from_args(args: &[String]) -> Source {
        if args.iter().any(|a| a == "--head") {
            Source::Head
        } else if args.iter().any(|a| a == "--staged") {
            Source::Staged
        } else {
            Source::Worktree
        }
    }

    pub fn paths(self) -> Result<Vec<String>, String> {
        match self {
            Source::Worktree => tracked(),
            Source::Staged => staged(),
            Source::Head => split_names(&git(&["ls-tree", "-r", "-z", "--name-only", "HEAD"])?),
        }
    }

    /// Inhalt einer Datei; im Arbeitsordner gelöschte Dateien sind leer.
    pub fn content(self, path: &str) -> Result<Vec<u8>, String> {
        match self {
            Source::Worktree => Ok(worktree_content(path).unwrap_or_default()),
            Source::Staged => staged_content(path),
            Source::Head => git(&["show", &format!("HEAD:{path}")]),
        }
    }
}

/// Ordner des Cargo-Workspace ab der Repo-Wurzel, mit `/` am Ende (leer an der Wurzel). Das Werkzeug wird im
/// Workspace-Ordner gestartet (`cargo xtask …`); in Desk ist das `app/src-tauri/`.
pub fn workspace_prefix() -> Result<String, String> {
    let raw = git(&["rev-parse", "--show-prefix"])?;
    Ok(String::from_utf8_lossy(&raw).trim().to_string())
}

/// Wurzel des Repositorys (alle Pfade der Prüfungen gelten ab hier).
pub fn toplevel() -> Result<std::path::PathBuf, String> {
    let raw = git(&["rev-parse", "--show-toplevel"])?;
    Ok(std::path::PathBuf::from(
        String::from_utf8_lossy(&raw).trim(),
    ))
}
