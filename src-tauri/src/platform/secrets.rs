use zeroize::Zeroizing;

const SERVICE: &str = "fr.vocalnotes.desktop";
const USER: &str = "mistral";

// No file/env import, no in-memory fallback, no Debug or provider error propagation.
fn entry() -> Result<keyring::Entry, &'static str> {
    keyring::Entry::new(SERVICE, USER).map_err(|_| "Trousseau indisponible.")
}

pub fn set(secret: String) -> Result<(), &'static str> {
    let secret = Zeroizing::new(secret);
    if secret.trim().is_empty() || secret.len() > 4096 || secret.contains(['\n', '\r']) {
        return Err("Clé invalide.");
    }
    entry()?
        .set_password(&secret)
        .map_err(|_| "Écriture du trousseau impossible.")
}

pub fn delete() -> Result<(), &'static str> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Suppression du trousseau impossible."),
    }
}

pub fn presence() -> Result<bool, &'static str> {
    match entry()?.get_password() {
        Ok(value) => {
            let _value = Zeroizing::new(value);
            Ok(true)
        }
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(_) => Err("Lecture du trousseau impossible."),
    }
}

// Only the native transports can obtain a secret; no IPC command returns it.
pub(crate) fn load() -> Result<Zeroizing<String>, &'static str> {
    entry()?
        .get_password()
        .map(Zeroizing::new)
        .map_err(|_| "Clé absente ou trousseau indisponible.")
}
