use super::{
    markdown_file::{DiskVersion, NativeFileVersion, SaveError},
    note_document::MAX_DOCUMENT_BYTES,
};
use crate::app::document::DocumentErrorCode;
use sha2::{Digest, Sha256};
use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    mem::{MaybeUninit, size_of},
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::AsRawHandle},
    path::{Component, Path, PathBuf},
    ptr,
    sync::atomic::{AtomicU64, Ordering},
};
use windows_sys::Win32::{Security as security, Storage::FileSystem as native};

static NEXT_BACKUP: AtomicU64 = AtomicU64::new(0);

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(Some(0)).collect()
}

fn unsupported(message: &'static str) -> SaveError {
    SaveError::coded(DocumentErrorCode::UnsupportedStorage, message)
}

fn information(file: &File) -> io::Result<native::BY_HANDLE_FILE_INFORMATION> {
    let mut info = MaybeUninit::uninit();
    if unsafe { native::GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(unsafe { info.assume_init() })
    }
}

fn identity(file: &File) -> io::Result<(u64, [u8; 16])> {
    let mut info = MaybeUninit::<native::FILE_ID_INFO>::uninit();
    if unsafe {
        native::GetFileInformationByHandleEx(
            file.as_raw_handle(),
            native::FileIdInfo,
            info.as_mut_ptr().cast(),
            size_of::<native::FILE_ID_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let info = unsafe { info.assume_init() };
    Ok((info.VolumeSerialNumber, info.FileId.Identifier))
}

fn security_descriptor(file: &File) -> Result<Vec<u8>, SaveError> {
    let requested = security::OWNER_SECURITY_INFORMATION
        | security::GROUP_SECURITY_INFORMATION
        | security::DACL_SECURITY_INFORMATION;
    let mut needed = 0;
    unsafe {
        security::GetKernelObjectSecurity(
            file.as_raw_handle(),
            requested,
            ptr::null_mut(),
            0,
            &mut needed,
        );
    }
    if needed == 0 || needed > 65_536 {
        return Err(unsupported(
            "Descripteur de sécurité inaccessible ou non pris en charge.",
        ));
    }
    let mut bytes = vec![0u8; needed as usize];
    if unsafe {
        security::GetKernelObjectSecurity(
            file.as_raw_handle(),
            requested,
            bytes.as_mut_ptr().cast(),
            bytes.len() as u32,
            &mut needed,
        )
    } == 0
    {
        return Err(unsupported("Lecture des permissions Windows impossible."));
    }
    bytes.truncate(needed as usize);
    Ok(bytes)
}

fn security_hash(file: &File) -> Result<[u8; 32], SaveError> {
    Ok(Sha256::digest(security_descriptor(file)?).into())
}

fn ordinary(info: &native::BY_HANDLE_FILE_INFORMATION, directory: bool) -> bool {
    let refused = native::FILE_ATTRIBUTE_REPARSE_POINT
        | native::FILE_ATTRIBUTE_DEVICE
        | native::FILE_ATTRIBUTE_OFFLINE
        | native::FILE_ATTRIBUTE_ENCRYPTED
        | native::FILE_ATTRIBUTE_SPARSE_FILE;
    info.dwFileAttributes & refused == 0
        && (info.dwFileAttributes & native::FILE_ATTRIBUTE_DIRECTORY != 0) == directory
}

fn local_ntfs(file: &File) -> Result<(), SaveError> {
    let mut name = [0u16; 32];
    if unsafe {
        native::GetVolumeInformationByHandleW(
            file.as_raw_handle(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            name.as_mut_ptr(),
            name.len() as u32,
        )
    } == 0
        || String::from_utf16_lossy(
            &name[..name.iter().position(|v| *v == 0).unwrap_or(name.len())],
        ) != "NTFS"
    {
        return Err(unsupported(
            "Une destination NTFS locale ordinaire est requise.",
        ));
    }
    let mut actual = vec![0u16; 4097];
    let count = unsafe {
        native::GetFinalPathNameByHandleW(
            file.as_raw_handle(),
            actual.as_mut_ptr(),
            actual.len() as u32,
            0,
        )
    } as usize;
    if count == 0 || count >= actual.len() {
        return Err(unsupported("Chemin effectif du volume inaccessible."));
    }
    let path = String::from_utf16_lossy(&actual[..count]);
    let bytes = path.as_bytes();
    if !path.starts_with(r"\\?\") || bytes.len() < 7 || bytes[5] != b':' || bytes[6] != b'\\' {
        return Err(unsupported(
            "Volume distant ou identité de volume non prise en charge.",
        ));
    }
    let root = wide(OsStr::new(&path[4..7]));
    // GetDriveTypeW: fixed disk = 3, removable disk = 2.
    if !matches!(unsafe { native::GetDriveTypeW(root.as_ptr()) }, 2 | 3) {
        return Err(unsupported("Volume réseau ou non local refusé."));
    }
    Ok(())
}

pub(super) fn open_directory(path: &Path) -> Result<File, SaveError> {
    let mut prefix = PathBuf::new();
    let mut last = None;
    for component in path.components() {
        prefix.push(component);
        if matches!(component, Component::Prefix(_)) {
            continue;
        }
        let file = OpenOptions::new()
            .access_mode(native::FILE_READ_ATTRIBUTES | native::READ_CONTROL)
            .share_mode(
                native::FILE_SHARE_READ | native::FILE_SHARE_WRITE | native::FILE_SHARE_DELETE,
            )
            .custom_flags(native::FILE_FLAG_BACKUP_SEMANTICS | native::FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&prefix)
            .map_err(|_| SaveError::new("Dossier Windows inaccessible."))?;
        if !ordinary(
            &information(&file).map_err(|_| unsupported("Identité du dossier inaccessible."))?,
            true,
        ) {
            return Err(unsupported(
                "Lien ou point de réanalyse dans la destination.",
            ));
        }
        local_ntfs(&file)?;
        last = Some(file);
    }
    let verified = last.ok_or_else(|| unsupported("Dossier local absolu requis."))?;
    let file = OpenOptions::new()
        .access_mode(native::FILE_GENERIC_READ | native::FILE_GENERIC_WRITE)
        .share_mode(native::FILE_SHARE_READ | native::FILE_SHARE_WRITE | native::FILE_SHARE_DELETE)
        .custom_flags(native::FILE_FLAG_BACKUP_SEMANTICS | native::FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| SaveError::new("Dossier non accessible en écriture."))?;
    let actual = identity(&file).map_err(|_| unsupported("Identité du dossier inaccessible."))?;
    let expected =
        identity(&verified).map_err(|_| unsupported("Identité du dossier inaccessible."))?;
    if actual != expected
        || !ordinary(
            &information(&file).map_err(|_| unsupported("Dossier inaccessible."))?,
            true,
        )
    {
        return Err(unsupported("Dossier remplacé pendant son ouverture."));
    }
    Ok(file)
}

pub(super) fn still_named(directory: &File, path: &Path) -> Result<bool, SaveError> {
    let current = open_directory(path)?;
    let before = identity(directory).map_err(|_| unsupported("Identité du dossier perdue."))?;
    let now = identity(&current).map_err(|_| unsupported("Identité du dossier inaccessible."))?;
    Ok(before == now)
}

pub(super) fn open_file(path: &Path, create: bool) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(create)
        .access_mode(
            native::FILE_GENERIC_READ
                | if create {
                    native::FILE_GENERIC_WRITE
                } else {
                    0
                },
        )
        .share_mode(native::FILE_SHARE_READ | native::FILE_SHARE_DELETE)
        .create_new(create)
        .custom_flags(native::FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    if !ordinary(&information(&file)?, false)
        || unsafe { native::GetFileType(file.as_raw_handle()) } != native::FILE_TYPE_DISK
    {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "ordinary disk file required",
        ));
    }
    Ok(file)
}

pub(super) fn read(file: &mut File) -> Result<(DiskVersion, Vec<u8>), SaveError> {
    let before = information(file).map_err(|_| unsupported("Identité disque inaccessible."))?;
    let (volume, file_id) =
        identity(file).map_err(|_| unsupported("Identité native indisponible."))?;
    let permissions = security_hash(file)?;
    let size = (u64::from(before.nFileSizeHigh) << 32) | u64::from(before.nFileSizeLow);
    if !ordinary(&before, false) || size > MAX_DOCUMENT_BYTES as u64 {
        return Err(SaveError::coded(
            DocumentErrorCode::SizeLimit,
            "Note ordinaire de moins de 1 Mio requise.",
        ));
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|_| SaveError::new("Lecture impossible."))?;
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take((MAX_DOCUMENT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| SaveError::new("Lecture impossible."))?;
    let after = information(file).map_err(|_| unsupported("Identité disque inaccessible."))?;
    if bytes.len() as u64 != size
        || bytes.len() > MAX_DOCUMENT_BYTES
        || before.nFileSizeLow != after.nFileSizeLow
        || before.nFileSizeHigh != after.nFileSizeHigh
        || before.ftLastWriteTime.dwHighDateTime != after.ftLastWriteTime.dwHighDateTime
        || before.ftLastWriteTime.dwLowDateTime != after.ftLastWriteTime.dwLowDateTime
        || before.dwFileAttributes != after.dwFileAttributes
        || before.nNumberOfLinks != after.nNumberOfLinks
        || identity(file).map_err(|_| unsupported("Identité disque inaccessible."))?
            != (volume, file_id)
        || security_hash(file)? != permissions
    {
        return Err(SaveError::coded(
            DocumentErrorCode::ExternalConflict,
            "Fichier modifié pendant lecture.",
        ));
    }
    Ok((
        DiskVersion {
            hash: Sha256::digest(&bytes).into(),
            native: NativeFileVersion::Windows {
                volume,
                file_id,
                links: after.nNumberOfLinks,
                attributes: after.dwFileAttributes,
                security_hash: permissions,
            },
        },
        bytes,
    ))
}

pub(super) fn install_absent(stage: &Path, destination: &Path) -> io::Result<()> {
    let from = wide(stage.as_os_str());
    let to = wide(destination.as_os_str());
    if unsafe { native::MoveFileExW(from.as_ptr(), to.as_ptr(), native::MOVEFILE_WRITE_THROUGH) }
        == 0
    {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(super) fn replace(stage: &Path, destination: &Path) -> Result<PathBuf, SaveError> {
    let mut original = open_file(destination, false).map_err(|_| {
        SaveError::retained(
            "Destination inaccessible ; préparation conservée.",
            stage.into(),
        )
    })?;
    let current = read(&mut original)?.0;
    if !replace_allowed(&current) {
        return Err(SaveError {
            code: DocumentErrorCode::UnsupportedStorage,
            message: "Liens ou attributs incompatibles avec un remplacement contrôlé.",
            recovery_paths: vec![stage.into()],
        });
    }
    let mut descriptor = security_descriptor(&original)?;
    let mut control = 0;
    let mut revision = 0;
    if unsafe {
        security::GetSecurityDescriptorControl(
            descriptor.as_mut_ptr().cast(),
            &mut control,
            &mut revision,
        )
    } == 0
    {
        return Err(unsupported("Contrôle des permissions Windows impossible."));
    }
    if control & security::SE_DACL_PROTECTED != 0 {
        return Err(SaveError {
            code: DocumentErrorCode::UnsupportedStorage,
            message: "DACL protégée non préservable par ce remplacement ; choisir Enregistrer sous.",
            recovery_paths: vec![stage.into()],
        });
    }
    let prepared = open_file(stage, false).map_err(|_| {
        SaveError::retained("Permissions de préparation inaccessibles.", stage.into())
    })?;
    if security_hash(&prepared)? != security_hash(&original)? {
        return Err(SaveError {
            code: DocumentErrorCode::UnsupportedStorage,
            message: "Permissions Windows non préservables ; destination inchangée.",
            recovery_paths: vec![stage.into()],
        });
    }
    drop(prepared);
    drop(original);
    let backup = stage.with_file_name(format!(
        "vocalnotes-displaced-{}-{}-{}.von",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| SaveError::new("Horloge indisponible ; remplacement abandonné."))?
            .as_nanos(),
        NEXT_BACKUP.fetch_add(1, Ordering::Relaxed),
    ));
    match std::fs::symlink_metadata(&backup) {
        Ok(_) => {
            return Err(SaveError::retained(
                "Nom de récupération déjà occupé.",
                stage.into(),
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(_) => {
            return Err(SaveError::retained(
                "Nom de récupération inaccessible.",
                stage.into(),
            ));
        }
    }
    let target = wide(destination.as_os_str());
    let replacement = wide(stage.as_os_str());
    let old = wide(backup.as_os_str());
    // No IGNORE_* flags and no unsupported REPLACEFILE_WRITE_THROUGH guarantee.
    if unsafe {
        native::ReplaceFileW(
            target.as_ptr(),
            replacement.as_ptr(),
            old.as_ptr(),
            0,
            ptr::null(),
            ptr::null(),
        )
    } == 0
    {
        let mut error = SaveError::new(
            "Remplacement Windows incomplet ou refusé ; vérifier les versions conservées.",
        );
        for path in [stage, backup.as_path(), destination] {
            match std::fs::symlink_metadata(path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                _ => error.recovery_paths.push(path.into()),
            }
        }
        Err(error)
    } else {
        Ok(backup)
    }
}

pub(super) fn replace_allowed(version: &DiskVersion) -> bool {
    matches!(&version.native, NativeFileVersion::Windows { links: 1, attributes, .. }
        if attributes & (native::FILE_ATTRIBUTE_READONLY | native::FILE_ATTRIBUTE_COMPRESSED
            | native::FILE_ATTRIBUTE_REPARSE_POINT | native::FILE_ATTRIBUTE_ENCRYPTED
            | native::FILE_ATTRIBUTE_OFFLINE | native::FILE_ATTRIBUTE_SPARSE_FILE) == 0)
}

pub(super) fn remove_verified(path: &Path, expected: &DiskVersion) -> Result<(), SaveError> {
    let mut file = OpenOptions::new()
        .access_mode(native::FILE_GENERIC_READ | native::DELETE)
        .share_mode(native::FILE_SHARE_READ | native::FILE_SHARE_DELETE)
        .custom_flags(native::FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| {
            SaveError::retained(
                "Ancienne version conservée ; nettoyage refusé.",
                path.into(),
            )
        })?;
    if &read(&mut file)?.0 != expected {
        return Err(SaveError::retained(
            "Version de récupération modifiée ; fichier conservé.",
            path.into(),
        ));
    }
    let info = native::FILE_DISPOSITION_INFO { DeleteFile: true };
    if unsafe {
        native::SetFileInformationByHandle(
            file.as_raw_handle(),
            native::FileDispositionInfo,
            (&info as *const native::FILE_DISPOSITION_INFO).cast(),
            size_of::<native::FILE_DISPOSITION_INFO>() as u32,
        )
    } == 0
    {
        Err(SaveError::retained(
            "Version déplacée conservée ; nettoyage impossible.",
            path.into(),
        ))
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "vocalnotes-win32-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn load(path: &Path) -> (DiskVersion, Vec<u8>) {
        read(&mut open_file(path, false).unwrap()).unwrap()
    }

    #[test]
    fn native_versions_distinguish_equal_bytes_and_detect_parent_replacement() {
        let f = Fixture::new();
        let old = open_directory(&f.0).unwrap();
        fs::write(f.path("first.von"), "same").unwrap();
        fs::write(f.path("second.von"), "same").unwrap();
        assert_ne!(load(&f.path("first.von")).0, load(&f.path("second.von")).0);
        assert!(still_named(&old, &f.0).unwrap());
        let moved = f.path("moved");
        fs::create_dir(&moved).unwrap();
        assert!(!still_named(&old, &moved).unwrap());
    }

    #[test]
    fn replace_keeps_displaced_identity_and_security_and_cleanup_is_handle_bound() {
        let f = Fixture::new();
        let target = f.path("note.von");
        let stage = f.path("prepared.von");
        fs::write(&target, "old\r\n").unwrap();
        fs::write(&stage, "new\r\n").unwrap();
        let old = load(&target).0;
        let NativeFileVersion::Windows {
            security_hash: old_security,
            ..
        } = old.native
        else {
            unreachable!()
        };
        let backup = replace(&stage, &target).unwrap();
        assert_eq!(load(&backup).0, old);
        assert_eq!(load(&target).1, b"new\r\n");
        let NativeFileVersion::Windows {
            security_hash: new_security,
            ..
        } = load(&target).0.native
        else {
            unreachable!()
        };
        assert_eq!(old_security, new_security);
        remove_verified(&backup, &old).unwrap();
        assert!(!backup.exists());
        assert!(!stage.exists());
    }

    #[test]
    fn absent_installation_cannot_overwrite_and_changed_recovery_is_never_deleted() {
        let f = Fixture::new();
        let target = f.path("note.von");
        let stage = f.path("prepared.von");
        fs::write(&target, "external").unwrap();
        fs::write(&stage, "local").unwrap();
        assert!(install_absent(&stage, &target).is_err());
        assert_eq!(load(&target).1, b"external");
        let version = load(&stage).0;
        fs::write(&stage, "changed recovery").unwrap();
        assert!(remove_verified(&stage, &version).is_err());
        assert_eq!(load(&stage).1, b"changed recovery");
        let destination = f.path("absent.von");
        install_absent(&stage, &destination).unwrap();
        assert_eq!(load(&destination).1, b"changed recovery");
    }

    #[test]
    fn hardlinks_and_readonly_destinations_refuse_replacement_without_adopting_preparation() {
        let f = Fixture::new();
        let target = f.path("note.von");
        let stage = f.path("prepared.von");
        fs::write(&target, "old").unwrap();
        fs::write(&stage, "new").unwrap();
        fs::hard_link(&target, f.path("alias.von")).unwrap();
        assert_eq!(
            replace(&stage, &target).unwrap_err().code,
            DocumentErrorCode::UnsupportedStorage
        );
        assert_eq!(load(&target).1, b"old");
        assert_eq!(load(&stage).1, b"new");
        fs::remove_file(f.path("alias.von")).unwrap();
        let mut permissions = fs::metadata(&target).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&target, permissions.clone()).unwrap();
        let result = replace(&stage, &target);
        permissions.set_readonly(false);
        fs::set_permissions(&target, permissions).unwrap();
        assert_eq!(
            result.unwrap_err().code,
            DocumentErrorCode::UnsupportedStorage
        );
        assert_eq!(load(&target).1, b"old");
    }

    #[test]
    fn protected_dacl_is_refused_before_any_destination_replacement() {
        let f = Fixture::new();
        let target = f.path("protected.von");
        let stage = f.path("prepared.von");
        fs::write(&target, "old").unwrap();
        fs::write(&stage, "new").unwrap();
        let file = OpenOptions::new()
            .access_mode(native::FILE_GENERIC_READ | native::WRITE_DAC)
            .open(&target)
            .unwrap();
        let mut descriptor = security_descriptor(&file).unwrap();
        assert_ne!(
            unsafe {
                security::SetSecurityDescriptorControl(
                    descriptor.as_mut_ptr().cast(),
                    security::SE_DACL_PROTECTED,
                    security::SE_DACL_PROTECTED,
                )
            },
            0
        );
        assert_ne!(
            unsafe {
                security::SetKernelObjectSecurity(
                    file.as_raw_handle(),
                    security::DACL_SECURITY_INFORMATION,
                    descriptor.as_mut_ptr().cast(),
                )
            },
            0
        );
        let mut control = 0;
        let mut revision = 0;
        let mut actual_descriptor = security_descriptor(&file).unwrap();
        assert_ne!(
            unsafe {
                security::GetSecurityDescriptorControl(
                    actual_descriptor.as_mut_ptr().cast(),
                    &mut control,
                    &mut revision,
                )
            },
            0
        );
        assert_ne!(control & security::SE_DACL_PROTECTED, 0);
        drop(file);
        let version = load(&target).0;
        assert_eq!(
            replace(&stage, &target).unwrap_err().code,
            DocumentErrorCode::UnsupportedStorage
        );
        assert_eq!(load(&target).0, version);
        assert_eq!(load(&stage).1, b"new");
    }
}
