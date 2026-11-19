//! TOOLSHELP - opens the help page in the default browser.
//!
//! The page is built into the plugin, written to the temp folder and opened from there, so it
//! works wherever the plugin is installed.

use ocs_plugin_api::host::HostApi;
use std::path::Path;

pub const HTML: &str = include_str!("../help/ocs_tools_help.html");

pub fn run(host: &mut dyn HostApi) {
    let path = std::env::temp_dir().join("ocs-tools-help.html");
    if let Err(e) = std::fs::write(&path, HTML) {
        host.push_error(&format!("TOOLSHELP: could not write {}: {e}", path.display()));
        return;
    }
    match open(&path) {
        Ok(()) => host.push_output(&format!("Help opened: {}", path.display())),
        Err(e) => host.push_error(&format!("TOOLSHELP: could not open {}: {e}", path.display())),
    }
}

#[cfg(windows)]
fn open(path: &Path) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    // CREATE_NO_WINDOW: no console flash.
    std::process::Command::new("cmd")
        .args(["/C", "start", ""])
        .arg(path)
        .creation_flags(0x0800_0000)
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "macos")]
fn open(path: &Path) -> std::io::Result<()> {
    std::process::Command::new("open").arg(path).spawn().map(|_| ())
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn open(path: &Path) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(path).spawn().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::HTML;

    /// Every command the plugin registers must be documented.
    #[test]
    fn help_page_documents_every_command() {
        let missing: Vec<&str> = crate::MANIFEST
            .command_prefixes
            .iter()
            .copied()
            .filter(|c| !HTML.contains(&format!("<code>{c}</code>")))
            .collect();
        assert!(missing.is_empty(), "not in the help page: {missing:?}");
    }

    #[test]
    fn help_page_is_well_formed_enough() {
        for tag in ["table", "ul", "h2"] {
            assert_eq!(
                HTML.matches(&format!("<{tag}")).count(),
                HTML.matches(&format!("</{tag}>")).count(),
                "unbalanced <{tag}>"
            );
        }
    }
}
