//! Only Markdown notes reach disk. Replacement is conditional and never rolls back
//! over an external writer. An interrupted preparation remains a recoverable note.
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
const LIMIT: usize = 1_048_576;
static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct SaveError {
    pub message: &'static str,
    pub recovery_paths: Vec<PathBuf>,
}
impl SaveError {
    fn new(message: &'static str) -> Self {
        Self {
            message,
            recovery_paths: vec![],
        }
    }
    fn retained(message: &'static str, path: PathBuf) -> Self {
        Self {
            message,
            recovery_paths: vec![path],
        }
    }
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
#[derive(Clone, PartialEq)]
struct Version {
    hash: [u8; 32],
    identity: (u64, u64),
    mode: u32,
    links: u64,
    owner: u32,
    group: u32,
}
pub struct MarkdownFile {
    path: PathBuf,
    version: Version,
    watcher: Option<File>,
}

impl MarkdownFile {
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
        let name = self.path.file_name().unwrap();
        let current = dir.read(name)?;
        let stage = dir.prepare(source.as_bytes())?;
        observe(SaveStage::Prepared).map_err(|_| {
            SaveError::retained(
                "Sauvegarde interrompue ; note préparée conservée.",
                stage.clone(),
            )
        })?;
        if current.0 != self.version {
            let external = dir.prepare(&current.1)?;
            return Ok(SaveOutcome::Conflict {
                local: stage,
                external,
            });
        }
        if current.0.links != 1 || !dir.replace_allowed(&self.path, &current.0)? {
            return Ok(SaveOutcome::Copy { path: stage });
        }
        dir.permissions(&stage, &current.0).map_err(|_| {
            SaveError::retained(
                "Permissions non préservées ; copie conservée.",
                stage.clone(),
            )
        })?;
        observe(SaveStage::BeforeExchange).map_err(|_| {
            SaveError::retained("Sauvegarde interrompue avant remplacement.", stage.clone())
        })?;
        if !dir.exchange(&stage, name).map_err(|_| {
            SaveError::retained(
                "Échange contrôlé impossible ; copie conservée.",
                stage.clone(),
            )
        })? {
            return Ok(SaveOutcome::Copy { path: stage });
        }
        // The staged path now holds the displaced disk version. Never delete it
        // on any uncertainty or perform a destructive rollback.
        observe(SaveStage::Exchanged).map_err(|_| {
            SaveError::retained(
                "Contrôle interrompu ; version disque déplacée conservée.",
                stage.clone(),
            )
        })?;
        dir.sync().map_err(|_| {
            SaveError::retained(
                "Durabilité non confirmée ; version disque déplacée conservée.",
                stage.clone(),
            )
        })?;
        let displaced = dir.read(stage.file_name().unwrap()).map_err(|_| {
            SaveError::retained(
                "Version déplacée inaccessible ; vérifier les deux notes.",
                stage.clone(),
            )
        })?;
        let installed = dir.read(name).map_err(|_| {
            SaveError::retained(
                "Destination modifiée pendant sauvegarde ; version déplacée conservée.",
                stage.clone(),
            )
        })?;
        let expected_hash: [u8; 32] = Sha256::digest(source.as_bytes()).into();
        if displaced.0 != self.version
            || installed.0.hash != expected_hash
            || !dir.still_named(&self.path)?
        {
            // Always preserve an explicit copy of our source even if the installed
            // name was changed a second time by another process.
            let local = dir.prepare(source.as_bytes())?;
            return Ok(SaveOutcome::Conflict {
                local,
                external: stage,
            });
        }
        dir.remove(&stage).map_err(|_| {
            SaveError::retained(
                "Note enregistrée ; ancienne version non supprimée.",
                stage.clone(),
            )
        })?;
        dir.sync()
            .map_err(|_| SaveError::new("Note écrite ; durabilité du dossier non confirmée."))?;
        self.version = installed.0;
        Ok(SaveOutcome::Saved)
    }
    pub fn create_copy(path: &Path, source: &str) -> Result<Self, SaveError> {
        if source.len() > LIMIT {
            return Err(SaveError::new("Note trop volumineuse."));
        }
        let dir = Directory::open(path)?;
        let mut file = dir.create(path.file_name().unwrap()).map_err(|_| {
            SaveError::new(
                "Création exclusive impossible : choisir un nouveau fichier Markdown local.",
            )
        })?;
        file.write_all(source.as_bytes())
            .and_then(|_| file.sync_all())
            .and_then(|_| dir.sync())
            .map_err(|_| {
                SaveError::retained(
                    "Écriture incomplète : note conservée en mémoire, vérifier la copie.",
                    path.into(),
                )
            })?;
        let (version, bytes) = dir.read(path.file_name().unwrap())?;
        if bytes != source.as_bytes() {
            return Err(SaveError::retained(
                "Copie modifiée pendant sauvegarde.",
                path.into(),
            ));
        }
        Ok(Self {
            path: path.into(),
            version,
            watcher: dir.watch(),
        })
    }
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
        if !note.is_absolute()
            || note.extension().and_then(|x| x.to_str()) != Some("md")
            || note
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
        {
            return Err(SaveError::new("Chemin absolu vers un fichier .md requis."));
        }
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
        #[cfg(not(target_os = "linux"))]
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
            let file = File::open(parent).map_err(|_| SaveError::new("Dossier inaccessible."))?;
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
        #[cfg(not(target_os = "linux"))]
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
        let mut file = self
            .open_file(name, false)
            .map_err(|_| SaveError::new("Note inaccessible ou lien symbolique."))?;
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
        #[cfg(unix)]
        let (identity, mode, links, owner, group) = {
            use std::os::unix::fs::MetadataExt;
            (
                (after.dev(), after.ino()),
                after.mode() & 0o7777,
                after.nlink(),
                after.uid(),
                after.gid(),
            )
        };
        #[cfg(not(unix))]
        let (identity, mode, links, owner, group) = ((0, 0), 0, 1, 0, 0);
        Ok((
            Version {
                hash: Sha256::digest(&bytes).into(),
                identity,
                mode,
                links,
                owner,
                group,
            },
            bytes,
        ))
    }
    fn prepare(&self, bytes: &[u8]) -> Result<PathBuf, SaveError> {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let name = format!(
            "vocalnotes-note-{}-{time}-{}.md",
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
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let f = self.open_file(path.file_name().unwrap(), false)?;
            #[cfg(target_os = "linux")]
            {
                use std::os::{fd::AsRawFd, unix::fs::MetadataExt};
                if f.metadata()?.gid() != version.group
                    && unsafe { libc::fchown(f.as_raw_fd(), u32::MAX, version.group) } != 0
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
            f.set_permissions(fs::Permissions::from_mode(version.mode))?;
            f.sync_all()
        }
        #[cfg(not(unix))]
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
            Ok(version.owner == unsafe { libc::geteuid() } && version.mode & 0o7000 == 0 && no_acl)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (path, version);
            Ok(false)
        }
    }
    fn exchange(&self, stage: &Path, name: &std::ffi::OsStr) -> io::Result<bool> {
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::AsRawFd;
            let a = cname(stage.file_name().unwrap()).map_err(|_| io::Error::other("name"))?;
            let b = cname(name).map_err(|_| io::Error::other("name"))?;
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
                return Ok(true);
            }
            let error = io::Error::last_os_error();
            if matches!(
                error.raw_os_error(),
                Some(libc::ENOSYS | libc::EINVAL | libc::EOPNOTSUPP)
            ) {
                Ok(false)
            } else {
                Err(error)
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (stage, name);
            Ok(false)
        }
    }
    fn remove(&self, path: &Path) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::AsRawFd;
            let name = cname(path.file_name().unwrap()).map_err(|_| io::Error::other("name"))?;
            if unsafe { libc::unlinkat(self.file.as_raw_fd(), name.as_ptr(), 0) } == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            fs::remove_file(path)
        }
    }
    fn still_named(&self, note: &Path) -> Result<bool, SaveError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let actual = fs::metadata(note.parent().unwrap())
                .map_err(|_| SaveError::new("Dossier déplacé pendant sauvegarde."))?;
            let expected = self
                .file
                .metadata()
                .map_err(|_| SaveError::new("Dossier inaccessible."))?;
            Ok(actual.dev() == expected.dev() && actual.ino() == expected.ino())
        }
        #[cfg(not(unix))]
        {
            let _ = note;
            Ok(true)
        }
    }
}
#[cfg(target_os = "linux")]
fn cname(name: &std::ffi::OsStr) -> Result<std::ffi::CString, SaveError> {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::CString::new(name.as_bytes()).map_err(|_| SaveError::new("Nom invalide."))
}
