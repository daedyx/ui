//! App-Kennung für Benachrichtigungen beim aktuellen Benutzer eintragen (Anzeigename im Info-Center), prüfen,
//! entfernen; dazu die eigenen Benachrichtigungen im Info-Center zählen und löschen (Proben und Aufräumen).
//! Nur `HKCU`, keine Adminrechte. Unter anderen Systemen tun die Funktionen nichts. `aumid` ist die App-Kennung der App
//! ([`crate::ids::AppIds::aumid`]).
use crate::ids::{AUMID_KEY_BASE, AUMID_NAME_VALUE};
use std::io;

fn key(aumid: &str) -> String {
    format!(r"{AUMID_KEY_BASE}\{aumid}")
}

#[cfg(windows)]
pub fn register(aumid: &str, display_name: &str) -> io::Result<()> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    let (k, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(key(aumid))?;
    k.set_value(AUMID_NAME_VALUE, &display_name)
}

#[cfg(windows)]
pub fn unregister(aumid: &str) -> io::Result<()> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    match RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(key(aumid)) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

#[cfg(windows)]
pub fn is_registered(aumid: &str) -> bool {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(key(aumid))
        .is_ok()
}

/// Wie viele Benachrichtigungen der App liegen gerade im Info-Center?
#[cfg(windows)]
pub fn history_count(aumid: &str) -> Option<u32> {
    use windows::UI::Notifications::ToastNotificationManager;
    use windows::core::HSTRING;
    let history = ToastNotificationManager::History().ok()?;
    history
        .GetHistoryWithId(&HSTRING::from(aumid))
        .ok()?
        .Size()
        .ok()
}

/// Löscht die Benachrichtigungen der App aus dem Info-Center.
#[cfg(windows)]
pub fn clear_history(aumid: &str) -> bool {
    use windows::UI::Notifications::ToastNotificationManager;
    use windows::core::HSTRING;
    ToastNotificationManager::History()
        .and_then(|h| h.ClearWithId(&HSTRING::from(aumid)))
        .is_ok()
}

#[cfg(not(windows))]
pub fn register(aumid: &str, _display_name: &str) -> io::Result<()> {
    let _ = key(aumid);
    Ok(())
}

#[cfg(not(windows))]
pub fn unregister(_aumid: &str) -> io::Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub fn is_registered(_aumid: &str) -> bool {
    false
}

#[cfg(not(windows))]
pub fn history_count(_aumid: &str) -> Option<u32> {
    None
}

#[cfg(not(windows))]
pub fn clear_history(_aumid: &str) -> bool {
    false
}
