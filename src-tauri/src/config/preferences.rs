//! Non-content settings only. No keys, transcripts or provider response payloads.
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
const LIMIT: u64 = 16_384;
static NEXT: AtomicU64 = AtomicU64::new(0);
pub const STT_MODEL: &str = "voxtral-mini-transcribe-realtime-3-0";
pub const LLM_MODEL: &str = "mistral-large-latest";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceSelection {
    pub provider: String,
    pub model: String,
}
impl ServiceSelection {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.provider != "mistral"
            || self.model.is_empty()
            || self.model.len() > 128
            || !self
                .model
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
        {
            return Err("Fournisseur ou identifiant de modèle non pris en charge.");
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelPreferences {
    pub stt: ServiceSelection,
    pub llm: ServiceSelection,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    pub stt: ServiceSelection,
    pub llm: ServiceSelection,
    pub last_note: Option<PathBuf>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            stt: ServiceSelection {
                provider: "mistral".into(),
                model: STT_MODEL.into(),
            },
            llm: ServiceSelection {
                provider: "mistral".into(),
                model: LLM_MODEL.into(),
            },
            last_note: None,
        }
    }
}
impl Preferences {
    fn validate(&self) -> Result<(), &'static str> {
        self.stt.validate()?;
        self.llm.validate()?;
        if let Some(path) = &self.last_note {
            if !path.is_absolute()
                || path.extension().and_then(|s| s.to_str()) != Some("md")
                || path.as_os_str().len() > 4096
                || path.to_string_lossy().contains('\0')
                || path
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err("Chemin de dernière note invalide.");
            }
        }
        Ok(())
    }
    pub fn models(&self) -> ModelPreferences {
        ModelPreferences {
            stt: self.stt.clone(),
            llm: self.llm.clone(),
        }
    }
}

pub struct PreferencesStore {
    path: Option<PathBuf>,
    value: Preferences,
}
impl PreferencesStore {
    pub fn memory() -> Self {
        Self {
            path: None,
            value: Preferences::default(),
        }
    }
    pub fn open(path: PathBuf) -> Result<Self, &'static str> {
        let value = match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Preferences::default(),
            Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() && meta.len() <= LIMIT => {
                let mut options = OpenOptions::new();
                options.read(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
                }
                let mut bytes = Vec::new();
                options
                    .open(&path)
                    .map_err(|_| "Lecture des préférences impossible.")?
                    .take(LIMIT + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|_| "Lecture des préférences impossible.")?;
                if bytes.len() as u64 > LIMIT {
                    return Err("Préférences trop volumineuses.");
                }
                serde_json::from_slice::<Preferences>(&bytes)
                    .map_err(|_| "Préférences invalides ; aucun remplacement automatique.")?
            }
            _ => return Err("Préférences inaccessibles ou fichier non ordinaire."),
        };
        value.validate()?;
        Ok(Self {
            path: Some(path),
            value,
        })
    }
    pub fn get(&self) -> Preferences {
        self.value.clone()
    }
    pub fn session_models(&self) -> Result<ModelPreferences, &'static str> {
        self.value.validate()?;
        Ok(self.value.models())
    }
    pub fn save(&mut self, next: Preferences) -> Result<(), &'static str> {
        next.validate()?;
        if let Some(path) = &self.path {
            write_atomic(path, &next)?;
        }
        self.value = next;
        Ok(())
    }
    pub fn set_models(&mut self, models: ModelPreferences) -> Result<Preferences, &'static str> {
        let mut next = self.get();
        next.stt = models.stt;
        next.llm = models.llm;
        self.save(next)?;
        Ok(self.get())
    }
    pub fn remember_note(&mut self, path: &Path) -> Result<(), &'static str> {
        if self.value.last_note.as_deref() == Some(path) {
            return Ok(());
        }
        let mut next = self.get();
        next.last_note = Some(path.into());
        self.save(next)
    }
}
fn write_atomic(path: &Path, value: &Preferences) -> Result<(), &'static str> {
    let parent = path.parent().ok_or("Dossier de préférences requis.")?;
    fs::create_dir_all(parent).map_err(|_| "Dossier de préférences indisponible.")?;
    if fs::symlink_metadata(parent)
        .map_err(|_| "Dossier de préférences indisponible.")?
        .file_type()
        .is_symlink()
    {
        return Err("Dossier de préférences symbolique refusé.");
    }
    match fs::symlink_metadata(path) {
        Ok(m) if !m.is_file() || m.file_type().is_symlink() => {
            return Err("Préférences non ordinaires : aucun remplacement.");
        }
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            return Err("Préférences inaccessibles.");
        }
        _ => {}
    }
    let temp = parent.join(format!(
        ".preferences-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temp)
        .map_err(|_| "Préparation des préférences impossible.")?;
    let bytes = serde_json::to_vec(value).map_err(|_| "Préférences invalides.")?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        #[cfg(unix)]
        fs::File::open(parent)?.sync_all()?;
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
        return Err("Enregistrement des préférences impossible ; réglages actifs conservés.");
    }
    Ok(())
}
