//! Only Markdown notes reach disk. Replacement is conditional and never rolls back
//! over an external writer. An interrupted preparation remains a recoverable note.
use crate::{
    app::document::{DocumentErrorCode, DocumentFormat, DocumentVersion},
    domain::intention::DraftingIntention,
};
use sha2::{Digest, Sha256};
#[cfg(not(windows))]
use std::fs;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
const LIMIT: usize = super::note_document::MAX_DOCUMENT_BYTES;
static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotePathError {
    InvalidPath,
    UnsupportedFormat,
}

impl NotePathError {
    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidPath => "Chemin absolu local sans traversal requis (4096 octets maximum).",
            Self::UnsupportedFormat => {
                "Choisissez une extension .von ou .md correspondant au format demandé."
            }
        }
    }
}

fn validate_local_path(path: &Path) -> Result<(), NotePathError> {
    let value = path.to_str().ok_or(NotePathError::InvalidPath)?;
    if value.len() > 4096
        || value.contains('\0')
        || value.starts_with("//")
        || !path.is_absolute()
        || path.file_name().is_none()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(NotePathError::InvalidPath);
    }
    #[cfg(windows)]
    {
        use std::path::Prefix;
        if !matches!(path.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_)))
            || path.components().any(|part| matches!(part, Component::Normal(name) if name.to_string_lossy().contains(':')))
        {
            return Err(NotePathError::InvalidPath);
        }
    }
    Ok(())
}

pub fn validate_note_path(path: &Path) -> Result<DocumentFormat, NotePathError> {
    validate_local_path(path)?;
    DocumentFormat::from_path(path).ok_or(NotePathError::UnsupportedFormat)
}

pub fn complete_note_path(path: &Path, format: DocumentFormat) -> Result<PathBuf, NotePathError> {
    validate_local_path(path)?;
    let completed = if path.extension().is_none() {
        path.with_extension(match format {
            DocumentFormat::Von => "von",
            DocumentFormat::Markdown => "md",
        })
    } else {
        path.to_owned()
    };
    if validate_note_path(&completed)? != format {
        return Err(NotePathError::UnsupportedFormat);
    }
    Ok(completed)
}

#[derive(Debug)]
pub struct SaveError {
    pub code: DocumentErrorCode,
    pub message: &'static str,
    pub recovery_paths: Vec<PathBuf>,
}
impl SaveError {
    pub(super) fn new(message: &'static str) -> Self {
        Self::coded(DocumentErrorCode::WriteFailed, message)
    }
    pub(super) fn coded(code: DocumentErrorCode, message: &'static str) -> Self {
        Self {
            code,
            message,
            recovery_paths: vec![],
        }
    }
    pub(super) fn retained(message: &'static str, path: PathBuf) -> Self {
        Self {
            code: DocumentErrorCode::WriteFailed,
            message,
            recovery_paths: vec![path],
        }
    }
}
impl From<NotePathError> for SaveError {
    fn from(error: NotePathError) -> Self {
        Self::coded(
            match error {
                NotePathError::InvalidPath => DocumentErrorCode::InvalidPath,
                NotePathError::UnsupportedFormat => DocumentErrorCode::UnsupportedFormat,
            },
            error.message(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeFileVersion {
    Linux {
        device: u64,
        inode: u64,
        mode: u32,
        links: u64,
        owner: u32,
        group: u32,
    },
    Windows {
        volume: u64,
        file_id: [u8; 16],
        links: u32,
        attributes: u32,
        security_hash: [u8; 32],
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiskVersion {
    pub(super) hash: [u8; 32],
    pub(super) native: NativeFileVersion,
}

impl DiskVersion {
    pub fn observe_optional(path: &Path) -> Result<Option<Self>, SaveError> {
        validate_note_path(path)?;
        Directory::open(path)?
            .read_optional(path.file_name().unwrap())
            .map(|value| value.map(|(version, _)| version))
    }

    pub fn observe(path: &Path) -> Result<Self, SaveError> {
        validate_note_path(path)?;
        Directory::open(path)?
            .read(path.file_name().unwrap())
            .map(|value| value.0)
    }

    fn links(&self) -> u64 {
        match self.native {
            NativeFileVersion::Linux { links, .. } => links,
            NativeFileVersion::Windows { links, .. } => u64::from(links),
        }
    }

    pub fn same_file(&self, other: &Self) -> bool {
        match (&self.native, &other.native) {
            (
                NativeFileVersion::Linux {
                    device: a,
                    inode: b,
                    ..
                },
                NativeFileVersion::Linux {
                    device: c,
                    inode: d,
                    ..
                },
            ) => (a, b) == (c, d),
            (
                NativeFileVersion::Windows {
                    volume: a,
                    file_id: b,
                    ..
                },
                NativeFileVersion::Windows {
                    volume: c,
                    file_id: d,
                    ..
                },
            ) => (a, b) == (c, d),
            _ => false,
        }
    }
}

#[derive(Clone)]
pub enum WriteContent {
    Document {
        source: String,
        intention: DraftingIntention,
    },
    MarkdownExport(String),
}

#[derive(Clone)]
pub struct WriteCandidate {
    content: WriteContent,
    bytes: Vec<u8>,
    path: PathBuf,
    format: DocumentFormat,
    note_version: DocumentVersion,
    expected: Option<DiskVersion>,
}

impl WriteCandidate {
    pub fn new(
        content: WriteContent,
        path: PathBuf,
        format: DocumentFormat,
        note_version: DocumentVersion,
        expected: Option<DiskVersion>,
    ) -> Result<Self, SaveError> {
        let path = complete_note_path(&path, format)?;
        let bytes = match &content {
            WriteContent::Document { source, intention } => {
                super::note_document::encode_checked(source, intention)
                    .map_err(|message| SaveError::coded(DocumentErrorCode::SizeLimit, message))?
                    .into_bytes()
            }
            WriteContent::MarkdownExport(source) => {
                if format != DocumentFormat::Markdown {
                    return Err(NotePathError::UnsupportedFormat.into());
                }
                if source.len() > LIMIT {
                    return Err(SaveError::coded(
                        DocumentErrorCode::SizeLimit,
                        "Export trop volumineux ; aucun fichier modifié.",
                    ));
                }
                source.as_bytes().to_owned()
            }
        };
        Ok(Self {
            content,
            bytes,
            path,
            format,
            note_version,
            expected,
        })
    }

    pub fn content(&self) -> &WriteContent {
        &self.content
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn format(&self) -> DocumentFormat {
        self.format
    }
    pub fn note_version(&self) -> DocumentVersion {
        self.note_version
    }
    pub fn expected_version(&self) -> Option<&DiskVersion> {
        self.expected.as_ref()
    }
    pub fn matches_disk(&self, observed: Option<&DiskVersion>) -> bool {
        self.expected.as_ref() == observed
    }
    pub fn with_overwrite_consent(mut self, consent: OverwriteConsent) -> Result<Self, SaveError> {
        if self.path != consent.path {
            return Err(SaveError::coded(
                DocumentErrorCode::ExternalConflict,
                "La confirmation ne correspond pas à cette destination.",
            ));
        }
        self.expected = Some(consent.version);
        Ok(self)
    }
    pub fn prepare(self) -> Result<PreparedWrite, SaveError> {
        let directory = Directory::open(&self.path)?;
        let path = directory.prepare_format(&self.bytes, self.format)?;
        let version = directory
            .read(path.file_name().unwrap())
            .map_err(|mut error| {
                error.recovery_paths.push(path.clone());
                error
            })?
            .0;
        Ok(PreparedWrite {
            candidate: self,
            _directory: directory,
            path,
            version,
        })
    }
}

pub struct OverwriteConsent {
    path: PathBuf,
    version: DiskVersion,
}

impl OverwriteConsent {
    pub fn new(path: PathBuf, version: DiskVersion) -> Result<Self, SaveError> {
        validate_note_path(&path)?;
        Ok(Self { path, version })
    }
}

pub struct PreparedWrite {
    candidate: WriteCandidate,
    // Keep the opened parent alive; dropping a preparation never deletes recovery data.
    _directory: Directory,
    path: PathBuf,
    version: DiskVersion,
}

impl PreparedWrite {
    pub fn candidate(&self) -> &WriteCandidate {
        &self.candidate
    }
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn install(self) -> WriteResult {
        self.install_observed(|_| Ok(()))
    }

    #[doc(hidden)]
    pub fn install_observed(
        self,
        mut observe: impl FnMut(SaveStage) -> io::Result<()>,
    ) -> WriteResult {
        let Self {
            candidate,
            _directory: directory,
            path,
            version,
        } = self;
        match directory.read(path.file_name().unwrap()) {
            Ok((actual, _)) if actual == version => {}
            Ok(_) => {
                return WriteResult::Failed {
                    error: SaveError::retained(
                        "Préparation modifiée depuis sa création ; destination inchangée.",
                        path,
                    ),
                    observed: None,
                };
            }
            Err(mut error) => {
                error.recovery_paths.push(path);
                return WriteResult::Failed {
                    error,
                    observed: None,
                };
            }
        }
        let result = if let Some(expected) = &candidate.expected {
            let mut file = MarkdownFile {
                path: candidate.path.clone(),
                version: expected.clone(),
                watcher: None,
            };
            match file.save_prepared(&candidate.bytes, &directory, path, &mut observe) {
                Ok(SaveOutcome::Saved) => Ok(file.version),
                Ok(SaveOutcome::Conflict { local, external }) => {
                    return Self::conflict(candidate, &directory, vec![local, external]);
                }
                Ok(SaveOutcome::Copy { path }) => Err(SaveError::retained(
                    "Copie conservée ; document non enregistré.",
                    path,
                )),
                Err(error) => Err(error),
            }
        } else {
            MarkdownFile::create_prepared(
                &candidate.path,
                &candidate.bytes,
                &directory,
                path,
                &mut observe,
            )
            .map(|file| file.version)
        };
        match result {
            Ok(version) => WriteResult::Saved { candidate, version },
            Err(error) if error.code == DocumentErrorCode::ExternalConflict => {
                Self::conflict(candidate, &directory, error.recovery_paths)
            }
            Err(error) => WriteResult::Failed {
                error,
                observed: None,
            },
        }
    }

    fn conflict(
        candidate: WriteCandidate,
        directory: &Directory,
        recovery_paths: Vec<PathBuf>,
    ) -> WriteResult {
        match directory.read_optional(candidate.path.file_name().unwrap()) {
            Ok(current) => WriteResult::Conflict {
                expected: candidate.expected,
                observed: current.map(|value| value.0),
                recovery_paths,
            },
            Err(mut error) => {
                error.recovery_paths.extend(recovery_paths);
                WriteResult::Failed {
                    error,
                    observed: None,
                }
            }
        }
    }
}

pub enum WriteResult {
    Saved {
        candidate: WriteCandidate,
        version: DiskVersion,
    },
    Conflict {
        expected: Option<DiskVersion>,
        observed: Option<DiskVersion>,
        recovery_paths: Vec<PathBuf>,
    },
    Failed {
        error: SaveError,
        observed: Option<DiskVersion>,
    },
}
#[derive(Debug, PartialEq)]
pub enum SaveOutcome {
    Saved,
    Conflict { local: PathBuf, external: PathBuf },
    Copy { path: PathBuf },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SaveStage {
    Prepared,
    BeforeExchange,
    Exchanged,
}
type Version = DiskVersion;
pub struct MarkdownFile {
    path: PathBuf,
    version: Version,
    watcher: Option<File>,
}

impl MarkdownFile {
    /// Adoption is memory-only: the installed version is the authority.
    pub fn from_saved(path: PathBuf, version: DiskVersion) -> Self {
        Self {
            path,
            version,
            watcher: None,
        }
    }

    pub fn version(&self) -> &DiskVersion {
        &self.version
    }

    pub fn open(path: &Path) -> Result<(Self, String), SaveError> {
        let dir = Directory::open(path)?;
        let (version, bytes) = dir.read(path.file_name().unwrap())?;
        let source =
            String::from_utf8(bytes).map_err(|_| SaveError::new("Lecture UTF-8 impossible."))?;
        Ok((
            Self {
                path: path.into(),
                version,
                watcher: dir.watch(),
            },
            source,
        ))
    }
    /// Watcher events are hints only. Content and identity are re-read before
    /// reporting a change; every save re-reads regardless of watcher state.
    pub fn changed_on_disk(&self) -> Result<bool, SaveError> {
        if let Some(watcher) = &self.watcher {
            let mut handle = watcher;
            let mut buffer = [0u8; 4096];
            let mut event = false;
            loop {
                match handle.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(_) => event = true,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(_) => {
                        event = true;
                        break;
                    }
                }
            }
            if !event {
                return Ok(false);
            }
        }
        let dir = Directory::open(&self.path)?;
        Ok(dir.read(self.path.file_name().unwrap())?.0 != self.version)
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn save(&mut self, source: &str) -> Result<SaveOutcome, SaveError> {
        self.save_observed(source, |_| Ok(()))
    }
    /// Observation points for deterministic filesystem race/interruption checks.
    /// The callback cannot bypass a native precondition or change saved content.
    #[doc(hidden)]
    pub fn save_observed(
        &mut self,
        source: &str,
        mut observe: impl FnMut(SaveStage) -> io::Result<()>,
    ) -> Result<SaveOutcome, SaveError> {
        if source.len() > LIMIT {
            return Err(SaveError::new("Note trop volumineuse."));
        }
        let dir = Directory::open(&self.path)?;
        let stage = dir.prepare_format(source.as_bytes(), validate_note_path(&self.path)?)?;
        self.save_prepared(source.as_bytes(), &dir, stage, &mut observe)
    }

    fn save_prepared(
        &mut self,
        source: &[u8],
        dir: &Directory,
        stage: PathBuf,
        mut observe: impl FnMut(SaveStage) -> io::Result<()>,
    ) -> Result<SaveOutcome, SaveError> {
        let format = validate_note_path(&self.path)?;
        let original_preparation = dir.read(stage.file_name().unwrap())?.0;
        let expected_hash: [u8; 32] = Sha256::digest(source).into();
        if original_preparation.hash != expected_hash || original_preparation.links() != 1 {
            return Err(SaveError::retained(
                "Préparation modifiée ; destination inchangée.",
                stage,
            ));
        }
        let name = self.path.file_name().unwrap();
        let current = dir
            .read_optional(name)
            .and_then(|current| {
                current.ok_or_else(|| {
                    SaveError::coded(
                        DocumentErrorCode::ExternalConflict,
                        "Destination supprimée depuis la dernière lecture.",
                    )
                })
            })
            .map_err(|mut error| {
                error.recovery_paths.push(stage.clone());
                error
            })?;
        observe(SaveStage::Prepared).map_err(|_| {
            SaveError::retained(
                "Sauvegarde interrompue ; note préparée conservée.",
                stage.clone(),
            )
        })?;
        if current.0 != self.version {
            let external = dir
                .prepare_format(&current.1, format)
                .map_err(|mut error| {
                    error.recovery_paths.push(stage.clone());
                    error
                })?;
            return Ok(SaveOutcome::Conflict {
                local: stage,
                external,
            });
        }
        if current.0.links() != 1 || !dir.replace_allowed(&self.path, &current.0)? {
            return Err(SaveError {
                code: DocumentErrorCode::UnsupportedStorage,
                message: "Liens ou permissions incompatibles avec un remplacement contrôlé.",
                recovery_paths: vec![stage],
            });
        }
        dir.permissions(&stage, &current.0).map_err(|_| {
            SaveError::retained(
                "Permissions non préservées ; copie conservée.",
                stage.clone(),
            )
        })?;
        let prepared = dir.read(stage.file_name().unwrap())?.0;
        if prepared.hash != expected_hash || !prepared.same_file(&original_preparation) {
            return Err(SaveError::retained(
                "Préparation modifiée pendant le contrôle ; destination inchangée.",
                stage,
            ));
        }
        observe(SaveStage::BeforeExchange).map_err(|_| {
            SaveError::retained("Sauvegarde interrompue avant remplacement.", stage.clone())
        })?;
        if !dir.still_named(&self.path)? || dir.read(stage.file_name().unwrap())?.0 != prepared {
            return Err(SaveError::retained(
                "Dossier ou préparation modifié ; remplacement abandonné.",
                stage,
            ));
        }
        let displaced_path = dir.exchange(&stage, name).map_err(|mut error| {
            if !error.recovery_paths.contains(&stage) {
                error.recovery_paths.push(stage.clone());
            }
            error
        })?;
        let checked: Result<SaveOutcome, SaveError> = (|| {
            observe(SaveStage::Exchanged).map_err(|_| {
                SaveError::retained(
                    "Contrôle interrompu ; version disque déplacée conservée.",
                    displaced_path.clone(),
                )
            })?;
            dir.sync().map_err(|_| {
                SaveError::retained(
                    "Durabilité non confirmée ; version disque déplacée conservée.",
                    displaced_path.clone(),
                )
            })?;
            let displaced = dir.read(displaced_path.file_name().unwrap()).map_err(|_| {
                SaveError::retained(
                    "Version déplacée inaccessible ; vérifier les deux notes.",
                    displaced_path.clone(),
                )
            })?;
            let installed = dir.read(name).map_err(|_| {
                SaveError::retained(
                    "Destination modifiée pendant sauvegarde ; version déplacée conservée.",
                    displaced_path.clone(),
                )
            })?;
            if displaced.0 != self.version
                || installed.0 != prepared
                || !dir.still_named(&self.path)?
            {
                // Always preserve an explicit copy of our source even if the installed
                // name was changed a second time by another process.
                let local = dir.prepare_format(source, format)?;
                return Ok(SaveOutcome::Conflict {
                    local,
                    external: displaced_path.clone(),
                });
            }
            dir.remove(&displaced_path, &displaced.0).map_err(|_| {
                SaveError::retained(
                    "Note enregistrée ; ancienne version non supprimée.",
                    displaced_path.clone(),
                )
            })?;
            dir.sync().map_err(|_| {
                SaveError::new("Note écrite ; durabilité du dossier non confirmée.")
            })?;
            self.version = installed.0;
            Ok(SaveOutcome::Saved)
        })();
        checked.map_err(|mut error| {
            for path in [&displaced_path, &self.path] {
                if !error.recovery_paths.contains(path) {
                    error.recovery_paths.push(path.clone());
                }
            }
            match dir.prepare_format(source, format) {
                Ok(local) => error.recovery_paths.push(local),
                Err(mut recovery_error) => {
                    recovery_error.recovery_paths.extend(error.recovery_paths);
                    return recovery_error;
                }
            }
            error
        })
    }
    pub fn create_copy(path: &Path, source: &str) -> Result<Self, SaveError> {
        Self::create_copy_observed(path, source, |_| Ok(()))
    }

    #[doc(hidden)]
    pub fn create_copy_observed(
        path: &Path,
        source: &str,
        mut observe: impl FnMut(SaveStage) -> io::Result<()>,
    ) -> Result<Self, SaveError> {
        if source.len() > LIMIT {
            return Err(SaveError::coded(
                DocumentErrorCode::SizeLimit,
                "Note trop volumineuse.",
            ));
        }
        let dir = Directory::open(path)?;
        let format = validate_note_path(path)?;
        let stage = dir.prepare_format(source.as_bytes(), format)?;
        Self::create_prepared(path, source.as_bytes(), &dir, stage, &mut observe)
    }

    fn create_prepared(
        path: &Path,
        source: &[u8],
        dir: &Directory,
        stage: PathBuf,
        mut observe: impl FnMut(SaveStage) -> io::Result<()>,
    ) -> Result<Self, SaveError> {
        let format = validate_note_path(path)?;
        let prepared = dir.read(stage.file_name().unwrap())?.0;
        let expected_hash: [u8; 32] = Sha256::digest(source).into();
        if prepared.hash != expected_hash {
            return Err(SaveError::retained(
                "Préparation modifiée ; création abandonnée.",
                stage,
            ));
        }
        observe(SaveStage::Prepared)
            .and_then(|_| observe(SaveStage::BeforeExchange))
            .map_err(|_| {
                SaveError::retained(
                    "Création interrompue ; préparation conservée.",
                    stage.clone(),
                )
            })?;
        if !dir.still_named(path)?
            || dir.read(stage.file_name().unwrap())?.0 != prepared
            || prepared.links() != 1
        {
            return Err(SaveError::retained(
                "Préparation ou dossier modifié ; création abandonnée.",
                stage,
            ));
        }
        dir.install_absent(&stage, path.file_name().unwrap())
            .map_err(|error| SaveError {
                code: if error.kind() == io::ErrorKind::AlreadyExists {
                    DocumentErrorCode::ExternalConflict
                } else if unsupported_install(&error) {
                    DocumentErrorCode::UnsupportedStorage
                } else {
                    DocumentErrorCode::WriteFailed
                },
                message: "Installation sans écrasement impossible ; préparation conservée.",
                recovery_paths: vec![stage.clone()],
            })?;
        let installed = (|| {
            observe(SaveStage::Exchanged)
                .map_err(|_| SaveError::new("Contrôle de création interrompu."))?;
            dir.sync()
                .map_err(|_| SaveError::new("Durabilité de création non confirmée."))?;
            let (version, _) = dir.read(path.file_name().unwrap())?;
            if version != prepared || !dir.still_named(path)? {
                return Err(SaveError::coded(
                    DocumentErrorCode::ExternalConflict,
                    "Destination modifiée après installation ; vérifier les versions.",
                ));
            }
            Ok(version)
        })();
        let version = match installed {
            Ok(version) => version,
            Err(mut error) => {
                error.recovery_paths.push(path.into());
                match dir.prepare_format(source, format) {
                    Ok(local) => error.recovery_paths.push(local),
                    Err(mut recovery_error) => {
                        recovery_error.recovery_paths.extend(error.recovery_paths);
                        return Err(recovery_error);
                    }
                }
                return Err(error);
            }
        };
        Ok(Self {
            path: path.into(),
            version,
            watcher: dir.watch(),
        })
    }
}

#[cfg(all(not(windows), not(target_os = "linux")))]
fn open_directory(path: &Path) -> io::Result<File> {
    File::open(path)
}

fn unsupported_install(error: &io::Error) -> bool {
    if error.kind() == io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(target_os = "linux")]
    return matches!(
        error.raw_os_error(),
        Some(libc::ENOSYS | libc::EINVAL | libc::EOPNOTSUPP)
    );
    #[cfg(not(target_os = "linux"))]
    false
}

struct Directory {
    path: PathBuf,
    file: File,
}
impl Directory {
    fn watch(&self) -> Option<File> {
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::{AsRawFd, FromRawFd};
            let fd = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
            if fd < 0 {
                return None;
            }
            // Own fd immediately so every failure closes it.
            let file = unsafe { File::from_raw_fd(fd) };
            let path =
                std::ffi::CString::new(format!("/proc/self/fd/{}", self.file.as_raw_fd())).ok()?;
            let mask = libc::IN_CLOSE_WRITE
                | libc::IN_MOVED_TO
                | libc::IN_MOVED_FROM
                | libc::IN_DELETE
                | libc::IN_ATTRIB
                | libc::IN_DELETE_SELF
                | libc::IN_MOVE_SELF;
            if unsafe { libc::inotify_add_watch(fd, path.as_ptr(), mask) } < 0 {
                None
            } else {
                Some(file)
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }
    fn open(note: &Path) -> Result<Self, SaveError> {
        validate_note_path(note)?;
        let parent = note
            .parent()
            .ok_or_else(|| SaveError::new("Dossier requis."))?;
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::{AsRawFd, FromRawFd};
            let mut file = File::open("/").map_err(|_| SaveError::new("Racine inaccessible."))?;
            for part in parent.components() {
                if let Component::Normal(name) = part {
                    let name = cname(name)?;
                    // SAFETY: valid C string, borrowed directory fd, returned fd is owned exactly once.
                    let fd = unsafe {
                        libc::openat(
                            file.as_raw_fd(),
                            name.as_ptr(),
                            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                        )
                    };
                    if fd < 0 {
                        return Err(SaveError::new(
                            "Dossier inaccessible ou lien symbolique : choisir une destination locale ordinaire.",
                        ));
                    }
                    file = unsafe { File::from_raw_fd(fd) };
                }
            }
            let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
            if unsafe { libc::fstatfs(file.as_raw_fd(), stat.as_mut_ptr()) } != 0 {
                return Err(SaveError::new("Type du volume inconnu."));
            }
            let kind = unsafe { stat.assume_init() }.f_type;
            // Local Linux filesystems used by desktops/tests; unknown or network = refuse.
            if ![
                0xEF53, 0x58465342, 0x9123683E, 0x01021994, 0x794c7630, 0x2FC12FC1,
            ]
            .contains(&kind)
            {
                return Err(SaveError::new(
                    "Volume non local ou inconnu : choisir un dossier local.",
                ));
            }
            Ok(Self {
                path: parent.into(),
                file,
            })
        }
        #[cfg(windows)]
        {
            Ok(Self {
                path: parent.into(),
                file: super::windows_file::open_directory(parent)?,
            })
        }
        #[cfg(all(not(windows), not(target_os = "linux")))]
        {
            let mut prefix = PathBuf::new();
            for part in parent.components() {
                prefix.push(part);
                if fs::symlink_metadata(&prefix)
                    .map_err(|_| SaveError::new("Dossier inaccessible."))?
                    .file_type()
                    .is_symlink()
                {
                    return Err(SaveError::new("Lien symbolique refusé."));
                }
            }
            let file =
                open_directory(parent).map_err(|_| SaveError::new("Dossier inaccessible."))?;
            Ok(Self {
                path: parent.into(),
                file,
            })
        }
    }
    fn open_file(&self, name: &std::ffi::OsStr, create: bool) -> io::Result<File> {
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::{AsRawFd, FromRawFd};
            use std::os::unix::ffi::OsStrExt;
            let name = std::ffi::CString::new(name.as_bytes())
                .map_err(|_| io::Error::other("invalid name"))?;
            let flags = libc::O_CLOEXEC
                | libc::O_NOFOLLOW
                | libc::O_NONBLOCK
                | if create {
                    libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL
                } else {
                    libc::O_RDONLY
                };
            let fd = unsafe { libc::openat(self.file.as_raw_fd(), name.as_ptr(), flags, 0o600) };
            if fd < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(unsafe { File::from_raw_fd(fd) })
            }
        }
        #[cfg(windows)]
        {
            super::windows_file::open_file(&self.path.join(name), create)
        }
        #[cfg(all(not(windows), not(target_os = "linux")))]
        {
            if create {
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(self.path.join(name))
            } else {
                let p = self.path.join(name);
                if fs::symlink_metadata(&p)?.file_type().is_symlink() {
                    return Err(io::Error::other("symlink"));
                }
                File::open(p)
            }
        }
    }
    fn create(&self, name: &std::ffi::OsStr) -> io::Result<File> {
        self.open_file(name, true)
    }
    fn read(&self, name: &std::ffi::OsStr) -> Result<(Version, Vec<u8>), SaveError> {
        self.read_optional(name)?
            .ok_or_else(|| SaveError::new("Note inaccessible."))
    }
    fn read_optional(
        &self,
        name: &std::ffi::OsStr,
    ) -> Result<Option<(Version, Vec<u8>)>, SaveError> {
        let file = match self.open_file(name, false) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(SaveError::new("Note inaccessible ou lien symbolique.")),
        };
        let value = self.read_file(file)?;
        if !self.still_named(&self.path.join(name))? {
            return Err(SaveError::coded(
                DocumentErrorCode::ExternalConflict,
                "Dossier déplacé pendant lecture.",
            ));
        }
        Ok(Some(value))
    }
    fn read_file(&self, mut file: File) -> Result<(Version, Vec<u8>), SaveError> {
        #[cfg(windows)]
        {
            super::windows_file::read(&mut file)
        }
        #[cfg(not(windows))]
        {
            let before = file
                .metadata()
                .map_err(|_| SaveError::new("Identité disque inaccessible."))?;
            if !before.is_file() || before.len() > LIMIT as u64 {
                return Err(SaveError::new("Note ordinaire de moins de 1 Mio requise."));
            }
            let mut bytes = Vec::new();
            std::io::Read::by_ref(&mut file)
                .take((LIMIT + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|_| SaveError::new("Lecture impossible."))?;
            let after = file
                .metadata()
                .map_err(|_| SaveError::new("Identité disque inaccessible."))?;
            if bytes.len() > LIMIT
                || before.len() != after.len()
                || before.modified().ok() != after.modified().ok()
            {
                return Err(SaveError::new("Fichier modifié pendant lecture."));
            }
            #[cfg(target_os = "linux")]
            let native = {
                use std::os::unix::fs::MetadataExt;
                NativeFileVersion::Linux {
                    device: after.dev(),
                    inode: after.ino(),
                    mode: after.mode() & 0o7777,
                    links: after.nlink(),
                    owner: after.uid(),
                    group: after.gid(),
                }
            };
            #[cfg(not(target_os = "linux"))]
            return Err(SaveError::coded(
                DocumentErrorCode::UnsupportedStorage,
                "Identité native indisponible.",
            ));
            #[cfg(target_os = "linux")]
            Ok((
                Version {
                    hash: Sha256::digest(&bytes).into(),
                    native,
                },
                bytes,
            ))
        }
    }
    fn prepare_format(&self, bytes: &[u8], format: DocumentFormat) -> Result<PathBuf, SaveError> {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let extension = match format {
            DocumentFormat::Von => "von",
            DocumentFormat::Markdown => "md",
        };
        let name = format!(
            "vocalnotes-note-{}-{time}-{}.{extension}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = self.path.join(&name);
        let mut file = self
            .create(std::ffi::OsStr::new(&name))
            .map_err(|_| SaveError::new("Préparation exclusive impossible ; note conservée."))?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .and_then(|_| self.sync())
            .map_err(|_| {
                SaveError::retained(
                    "Écriture incomplète ; note conservée en mémoire et copie à vérifier.",
                    path.clone(),
                )
            })?;
        Ok(path)
    }
    fn sync(&self) -> io::Result<()> {
        self.file.sync_all()
    }
    fn permissions(&self, path: &Path, version: &Version) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;
            let NativeFileVersion::Linux { mode, group, .. } = version.native else {
                return Err(io::Error::other("Linux identity required"));
            };
            let f = self.open_file(path.file_name().unwrap(), false)?;
            #[cfg(target_os = "linux")]
            {
                use std::os::{fd::AsRawFd, unix::fs::MetadataExt};
                if f.metadata()?.gid() != group
                    && unsafe { libc::fchown(f.as_raw_fd(), u32::MAX, group) } != 0
                {
                    return Err(io::Error::last_os_error());
                }
                // A new file can inherit a directory ACL absent from the original.
                if unsafe { libc::fremovexattr(f.as_raw_fd(), c"system.posix_acl_access".as_ptr()) }
                    != 0
                    && !matches!(
                        io::Error::last_os_error().raw_os_error(),
                        Some(libc::ENODATA | libc::EOPNOTSUPP)
                    )
                {
                    return Err(io::Error::last_os_error());
                }
            }
            f.set_permissions(fs::Permissions::from_mode(mode))?;
            f.sync_all()
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (path, version);
            Ok(())
        }
    }
    fn replace_allowed(&self, path: &Path, version: &Version) -> Result<bool, SaveError> {
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::AsRawFd;
            let f = self
                .open_file(path.file_name().unwrap(), false)
                .map_err(|_| SaveError::new("Destination inaccessible."))?;
            // Preserve ACL/ownership by declining replacement when plain mode bits are insufficient.
            let acl = unsafe {
                libc::fgetxattr(
                    f.as_raw_fd(),
                    c"system.posix_acl_access".as_ptr(),
                    std::ptr::null_mut(),
                    0,
                )
            };
            let no_acl = acl == 0
                || (acl < 0
                    && matches!(
                        io::Error::last_os_error().raw_os_error(),
                        Some(libc::ENODATA | libc::EOPNOTSUPP)
                    ));
            Ok(
                matches!(version.native, NativeFileVersion::Linux { owner, mode, .. }
                if owner == unsafe { libc::geteuid() } && mode & 0o7000 == 0 && no_acl),
            )
        }
        #[cfg(windows)]
        {
            let _ = path;
            Ok(super::windows_file::replace_allowed(version))
        }
        #[cfg(all(not(windows), not(target_os = "linux")))]
        {
            let _ = (path, version);
            Ok(false)
        }
    }
    fn exchange(&self, stage: &Path, name: &std::ffi::OsStr) -> Result<PathBuf, SaveError> {
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::AsRawFd;
            let a = cname(stage.file_name().unwrap())?;
            let b = cname(name)?;
            let result = unsafe {
                libc::renameat2(
                    self.file.as_raw_fd(),
                    a.as_ptr(),
                    self.file.as_raw_fd(),
                    b.as_ptr(),
                    libc::RENAME_EXCHANGE,
                )
            };
            if result == 0 {
                return Ok(stage.into());
            }
            let error = io::Error::last_os_error();
            if matches!(
                error.raw_os_error(),
                Some(libc::ENOSYS | libc::EINVAL | libc::EOPNOTSUPP)
            ) {
                Err(SaveError::coded(
                    DocumentErrorCode::UnsupportedStorage,
                    "Échange natif indisponible.",
                ))
            } else {
                Err(SaveError::new("Échange contrôlé impossible."))
            }
        }
        #[cfg(windows)]
        {
            super::windows_file::replace(stage, &self.path.join(name))
        }
        #[cfg(all(not(windows), not(target_os = "linux")))]
        {
            let _ = (stage, name);
            Err(SaveError::coded(
                DocumentErrorCode::UnsupportedStorage,
                "Échange natif indisponible.",
            ))
        }
    }
    fn remove(&self, path: &Path, expected: &DiskVersion) -> Result<(), SaveError> {
        #[cfg(windows)]
        {
            super::windows_file::remove_verified(path, expected)
        }
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::AsRawFd;
            if &self.read(path.file_name().unwrap())?.0 != expected {
                return Err(SaveError::retained(
                    "Version de récupération modifiée ; fichier conservé.",
                    path.into(),
                ));
            }
            let name = cname(path.file_name().unwrap())?;
            if unsafe { libc::unlinkat(self.file.as_raw_fd(), name.as_ptr(), 0) } == 0 {
                Ok(())
            } else {
                Err(SaveError::retained(
                    "Nettoyage refusé ; ancienne version conservée.",
                    path.into(),
                ))
            }
        }
        #[cfg(all(not(windows), not(target_os = "linux")))]
        {
            let _ = (path, expected);
            Err(SaveError::coded(
                DocumentErrorCode::UnsupportedStorage,
                "Nettoyage natif indisponible.",
            ))
        }
    }
    fn install_absent(&self, stage: &Path, name: &std::ffi::OsStr) -> io::Result<()> {
        #[cfg(windows)]
        {
            super::windows_file::install_absent(stage, &self.path.join(name))
        }
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::AsRawFd;
            let from = cname(stage.file_name().unwrap()).map_err(|_| io::Error::other("name"))?;
            let to = cname(name).map_err(|_| io::Error::other("name"))?;
            // NOREPLACE makes an intervening creation a conflict, never an overwrite.
            if unsafe {
                libc::renameat2(
                    self.file.as_raw_fd(),
                    from.as_ptr(),
                    self.file.as_raw_fd(),
                    to.as_ptr(),
                    libc::RENAME_NOREPLACE,
                )
            } == 0
            {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
        #[cfg(all(not(windows), not(target_os = "linux")))]
        {
            let _ = (stage, name);
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native installation unavailable",
            ))
        }
    }
    fn still_named(&self, note: &Path) -> Result<bool, SaveError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let actual = Directory::open(note)?
                .file
                .metadata()
                .map_err(|_| SaveError::new("Dossier déplacé pendant sauvegarde."))?;
            let expected = self
                .file
                .metadata()
                .map_err(|_| SaveError::new("Dossier inaccessible."))?;
            Ok(actual.dev() == expected.dev() && actual.ino() == expected.ino())
        }
        #[cfg(windows)]
        {
            let _ = note;
            super::windows_file::still_named(&self.file, &self.path)
        }
        #[cfg(all(not(unix), not(windows)))]
        {
            let _ = note;
            Err(SaveError::coded(
                DocumentErrorCode::UnsupportedStorage,
                "Identité du dossier indisponible.",
            ))
        }
    }
}
#[cfg(target_os = "linux")]
fn cname(name: &std::ffi::OsStr) -> Result<std::ffi::CString, SaveError> {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::CString::new(name.as_bytes()).map_err(|_| SaveError::new("Nom invalide."))
}

#[cfg(test)]
mod write_contract_tests {
    use super::*;
    use crate::{app::document::DocumentVersion, domain::intention::DraftingIntention};

    fn version(inode: u64, hash: u8) -> DiskVersion {
        DiskVersion {
            hash: [hash; 32],
            native: NativeFileVersion::Linux {
                device: 1,
                inode,
                mode: 0o600,
                links: 1,
                owner: 1000,
                group: 1000,
            },
        }
    }

    fn candidate(path: PathBuf, expected: Option<DiskVersion>) -> WriteCandidate {
        WriteCandidate::new(
            WriteContent::Document {
                source: "local".into(),
                intention: DraftingIntention::default(),
            },
            path,
            DocumentFormat::Von,
            DocumentVersion {
                note_id: 1,
                revision: 2,
                input_generation: 3,
            },
            expected,
        )
        .ok()
        .unwrap()
    }

    #[test]
    fn overwrite_consent_never_authorizes_another_path_or_disk_version() {
        let path = std::env::temp_dir().join("consent.von");
        let initial = version(1, 1);
        let external = version(2, 2);
        let candidate = candidate(path.clone(), Some(initial.clone()));
        assert!(candidate.matches_disk(Some(&initial)));
        assert!(!candidate.matches_disk(Some(&external)));
        assert!(!candidate.matches_disk(None));
        let consent = OverwriteConsent::new(path.clone(), external.clone()).unwrap();
        let candidate = candidate.with_overwrite_consent(consent).ok().unwrap();
        assert!(candidate.matches_disk(Some(&external)));
        assert!(!candidate.matches_disk(Some(&initial)));
        assert!(!candidate.matches_disk(Some(&version(3, 2))));
        assert!(!candidate.matches_disk(Some(&version(2, 3))));
        let mut metadata_changed = external.clone();
        let NativeFileVersion::Linux { mode, .. } = &mut metadata_changed.native else {
            unreachable!()
        };
        *mode = 0o644;
        assert!(!candidate.matches_disk(Some(&metadata_changed)));
        assert!(
            candidate
                .clone()
                .with_overwrite_consent(
                    OverwriteConsent::new(path.with_file_name("other.von"), external).unwrap()
                )
                .is_err()
        );
    }

    #[test]
    fn an_absent_destination_is_not_permission_to_replace_a_new_file() {
        let path = std::env::temp_dir().join("absent.von");
        let candidate = candidate(path, None);
        assert!(candidate.matches_disk(None));
        assert!(!candidate.matches_disk(Some(&version(1, 1))));
    }
}
