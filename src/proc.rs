//! Lancement de sous-processus sans ouvrir de fenêtre de console (Windows).

use std::ffi::OsStr;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut c = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    c.stdin(Stdio::null());
    c
}

/// Dossier de l'exécutable (c'est là que l'installateur dépose yt-dlp et ffmpeg).
pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(PathBuf::from)
}

/// Message compréhensible pour une erreur de lancement de programme.
pub fn explain(e: &io::Error) -> String {
    if e.kind() == io::ErrorKind::NotFound {
        return "introuvable".into();
    }
    // 4551 / 4552 : stratégie de contrôle d'application ; 1260 : stratégie de groupe ; 5 : accès refusé
    match e.raw_os_error() {
        Some(4551 | 4552 | 1260) => "bloqué par Windows (Contrôle intelligent des applications)".into(),
        Some(5) => "accès refusé".into(),
        _ => e.to_string(),
    }
}

/// Dernière ligne non vide d'une sortie (pour afficher l'erreur utile), tronquée.
pub fn last_line(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let line = text.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    line.chars().take(140).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explains_common_errors() {
        assert_eq!(explain(&io::Error::from(io::ErrorKind::NotFound)), "introuvable");
        assert!(explain(&io::Error::from_raw_os_error(4551)).contains("Windows"));
    }

    #[test]
    fn last_line_skips_blank_and_truncates() {
        assert_eq!(last_line(b"a\nERROR: boom\n\n"), "ERROR: boom");
        assert_eq!(last_line(&vec![b'x'; 500]).len(), 140);
    }
}
