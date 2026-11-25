//! Autostart über `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` (nur aktueller Benutzer, keine Adminrechte).
//! Der Wert ist `"<Programmpfad>" --hidden`. Der Task-Manager kann den Eintrag zusätzlich über
//! `StartupApproved\Run` deaktivieren; das lesen wir nur und überschreiben es nie.
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Off,
    On,
    /// Eintrag vorhanden, aber im Task-Manager deaktiviert.
    Blocked,
}

pub trait AutostartStore {
    fn state(&self) -> io::Result<State>;
    fn set(&mut self, on: bool) -> io::Result<()>;
}

/// Baut die Befehlszeile. Lehnt Pfade mit Anführungszeichen ab (würden die Zeile zerbrechen).
pub fn command_line(exe: &Path) -> io::Result<String> {
    let s = exe
        .to_str()
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    if s.contains('"') || s.is_empty() {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    Ok(format!("\"{s}\" {}", crate::ids::HIDDEN_ARG))
}

/// `StartupApproved`: erstes Byte gerade (02, 06) = erlaubt, ungerade (03, 07) = deaktiviert.
pub fn approved(bytes: Option<&[u8]>) -> bool {
    bytes
        .and_then(|b| b.first())
        .is_none_or(|first| first & 1 == 0)
}

/// Ersatz für Tests und für Systeme ohne Autostart.
#[derive(Debug, Default)]
pub struct FakeStore {
    pub on: bool,
    pub fail: bool,
}

impl AutostartStore for FakeStore {
    fn state(&self) -> io::Result<State> {
        Ok(if self.on { State::On } else { State::Off })
    }

    fn set(&mut self, on: bool) -> io::Result<()> {
        if self.fail {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        self.on = on;
        Ok(())
    }
}

#[cfg(windows)]
pub use win::RegistryStore;

#[cfg(windows)]
mod win {
    use super::*;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};

    /// Schlüsselpfade sind einstellbar, damit Tests nicht den echten `Run`-Schlüssel anfassen.
    pub struct RegistryStore {
        pub run_key: String,
        pub approved_key: String,
        pub value: String,
        pub command: String,
    }

    impl RegistryStore {
        /// `run_value`: Name des Autostart-Werts der App ([`crate::ids::AppIds::run_value`]).
        pub fn for_current_exe(run_value: &str) -> io::Result<Self> {
            let exe = std::env::current_exe()?;
            Ok(Self {
                run_key: crate::ids::RUN_KEY.to_string(),
                approved_key: crate::ids::APPROVED_KEY.to_string(),
                value: run_value.to_string(),
                command: command_line(&exe)?,
            })
        }

        fn hkcu() -> RegKey {
            RegKey::predef(HKEY_CURRENT_USER)
        }

        fn approved_bytes(&self) -> Option<Vec<u8>> {
            let key = Self::hkcu()
                .open_subkey_with_flags(&self.approved_key, KEY_READ)
                .ok()?;
            key.get_raw_value(&self.value)
                .ok()
                .map(|v| v.bytes.to_vec())
        }
    }

    impl AutostartStore for RegistryStore {
        fn state(&self) -> io::Result<State> {
            let key = match Self::hkcu().open_subkey_with_flags(&self.run_key, KEY_READ) {
                Ok(k) => k,
                Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(State::Off),
                Err(e) => return Err(e),
            };
            let present: io::Result<String> = key.get_value(&self.value);
            match present {
                Ok(_) if approved(self.approved_bytes().as_deref()) => Ok(State::On),
                Ok(_) => Ok(State::Blocked),
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(State::Off),
                Err(e) => Err(e),
            }
        }

        fn set(&mut self, on: bool) -> io::Result<()> {
            if on {
                let (key, _) = Self::hkcu().create_subkey(&self.run_key)?;
                key.set_value(&self.value, &self.command)
            } else {
                let key = match Self::hkcu().open_subkey_with_flags(&self.run_key, KEY_SET_VALUE) {
                    Ok(k) => k,
                    Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
                    Err(e) => return Err(e),
                };
                match key.delete_value(&self.value) {
                    Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
                    other => other,
                }
            }
        }
    }
}
