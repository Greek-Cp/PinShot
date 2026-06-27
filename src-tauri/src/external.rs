//! Opening a URL in the OS default browser — the one explicit, user-initiated
//! hand-off in the app (QR "Open URL", FR-029). PinShot never sends the
//! screenshot anywhere; this only launches a user-chosen `http(s)` link via the
//! OS, with no network request from the app itself.

/// Opens `url` in the default browser if it is a well-formed `http(s)` URL.
/// Rejects anything else (no `file://`, no shell metacharacters) and passes the
/// URL as a single process argument (never a shell string) to avoid injection.
pub fn open_url(url: &str) -> Result<(), String> {
    let lower = url.trim().to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err("invalid_url".to_string());
    }
    if url.chars().any(|c| c.is_control()) {
        return Err("invalid_url".to_string());
    }
    spawn(url).map_err(|e| format!("open_failed: {e}"))
}

/// Reveals a file in the OS file manager (Finder / Explorer), selecting it.
/// Used by the saved-image preview toast's "Show in Finder" action.
pub fn reveal(path: &str) -> Result<(), String> {
    reveal_impl(path).map_err(|e| format!("reveal_failed: {e}"))
}

#[cfg(target_os = "macos")]
fn reveal_impl(path: &str) -> std::io::Result<()> {
    std::process::Command::new("open")
        .args(["-R", path])
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn reveal_impl(path: &str) -> std::io::Result<()> {
    std::process::Command::new("explorer")
        .arg(format!("/select,{path}"))
        .spawn()?;
    Ok(())
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
fn reveal_impl(path: &str) -> std::io::Result<()> {
    let dir = std::path::Path::new(path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    std::process::Command::new("xdg-open").arg(dir).spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn spawn(url: &str) -> std::io::Result<()> {
    std::process::Command::new("open").arg(url).spawn()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn spawn(url: &str) -> std::io::Result<()> {
    // `start` is a cmd builtin; the empty "" is the (ignored) window title so a
    // quoted URL is not mistaken for it. The URL is a separate argument.
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()?;
    Ok(())
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
fn spawn(url: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_http_schemes() {
        assert_eq!(open_url("file:///etc/passwd").unwrap_err(), "invalid_url");
        assert_eq!(open_url("javascript:alert(1)").unwrap_err(), "invalid_url");
        assert_eq!(open_url("ftp://x").unwrap_err(), "invalid_url");
    }

    #[test]
    fn rejects_control_characters() {
        assert_eq!(open_url("https://x\n&calc").unwrap_err(), "invalid_url");
    }
}
