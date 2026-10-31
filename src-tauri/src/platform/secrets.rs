use zeroize::Zeroizing;
const SERVICE: &str = "fr.vocalnotes.desktop";
const USER: &str = "mistral";

// No global builder, no file/env import and no mock/memory fallback.
// Deliberately no Debug/Serialize implementation and no exposed secret getter.
pub struct SecretStore {
    entry: keyring::Entry,
}
impl SecretStore {
    pub fn native(service: &str, user: &str) -> Result<Self, &'static str> {
        #[cfg(target_os = "linux")]
        let credential =
            keyring::secret_service::SsCredential::new_with_target(None, service, user);
        #[cfg(target_os = "windows")]
        let credential = keyring::windows::WinCredential::new_with_target(None, service, user);
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            Self::from_entry(keyring::Entry::new_with_credential(Box::new(
                credential.map_err(|_| "Trousseau indisponible.")?,
            )))
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = (service, user);
            Err("Backend de trousseau non pris en charge.")
        }
    }
    pub fn from_entry(entry: keyring::Entry) -> Result<Self, &'static str> {
        #[cfg(target_os = "linux")]
        let native = entry
            .get_credential()
            .is::<keyring::secret_service::SsCredential>();
        #[cfg(target_os = "windows")]
        let native = entry
            .get_credential()
            .is::<keyring::windows::WinCredential>();
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        let native = false;
        if !native {
            return Err("Backend de trousseau non natif refusé.");
        }
        Ok(Self { entry })
    }
    pub fn set(&self, secret: String) -> Result<(), &'static str> {
        self.set_protected(Zeroizing::new(secret))
    }
    fn set_protected(&self, secret: Zeroizing<String>) -> Result<(), &'static str> {
        if secret.trim().is_empty() || secret.len() > 4096 || secret.contains(['\n', '\r', '\0']) {
            return Err("Clé invalide.");
        }
        self.entry
            .set_password(&secret)
            .map_err(|_| "Écriture du trousseau impossible.")
    }
    pub fn delete(&self) -> Result<(), &'static str> {
        match self.entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err("Suppression du trousseau impossible."),
        }
    }
    pub fn presence(&self) -> Result<bool, &'static str> {
        match self.entry.get_password() {
            Ok(value) => {
                let _value = Zeroizing::new(value);
                Ok(true)
            }
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(_) => Err("Lecture du trousseau impossible."),
        }
    }
    fn load(&self) -> Result<Zeroizing<String>, &'static str> {
        self.entry
            .get_password()
            .map(Zeroizing::new)
            .map_err(|_| "Clé absente ou trousseau indisponible.")
    }
}
fn app_store() -> Result<SecretStore, &'static str> {
    SecretStore::native(SERVICE, USER)
}
pub fn set(secret: String) -> Result<(), &'static str> {
    set_protected(Zeroizing::new(secret))
}
pub(crate) fn set_protected(secret: Zeroizing<String>) -> Result<(), &'static str> {
    app_store()?.set_protected(secret)
}
pub fn delete() -> Result<(), &'static str> {
    app_store()?.delete()
}
pub fn presence() -> Result<bool, &'static str> {
    app_store()?.presence()
}
pub(crate) fn load() -> Result<Zeroizing<String>, &'static str> {
    app_store()?.load()
}
