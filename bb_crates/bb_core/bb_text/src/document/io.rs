use std::{
    fs,
    io::{Seek, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use super::{Document, DocumentError};

/// UTF-8 byte order mark.
const BOM: [u8; 3] = [0xef, 0xbb, 0xbf];

impl Document {
    /// Open a UTF-8 file, preserving its BOM and line breaks.
    ///
    /// # Errors
    /// Returns a file or UTF-8 error.
    pub fn open(path: &Path) -> Result<Self, DocumentError> {
        let bytes: Vec<u8> = fs::read(path)?;
        let bom: bool = bytes.starts_with(&BOM);
        let text: String = String::from_utf8(if bom {
            bytes.get(3..).unwrap_or(&[]).to_vec()
        } else {
            bytes
        })?;
        let mut doc: Self = Self::from_text(&text, Some(path.to_path_buf()));
        doc.format.bom = bom;
        Ok(doc)
    }

    /// Refuse every later save with [`DocumentError::WriteLocked`]; edits
    /// stay possible in memory. There is no unlock.
    pub fn lock_writes(&mut self) {
        self.write_locked = true;
    }

    /// Whether saves are locked.
    #[must_use]
    pub fn writes_locked(&self) -> bool {
        self.write_locked
    }

    /// Save the complete file without losing the target's identity.
    ///
    /// The target (a symlink is followed) must be writable, or the save fails
    /// with [`DocumentError::ReadOnly`] and the file is unchanged. A file with
    /// more than one hard link, or one owned by another user or group, is
    /// rewritten in place. Any other file is written to a same-directory copy
    /// that carries the target's mode and extended attributes, then renamed.
    ///
    /// # Errors
    /// Returns [`DocumentError::WriteLocked`], [`DocumentError::ReadOnly`], a file
    /// error or [`DocumentError::NoPath`].
    pub fn save(&mut self) -> Result<(), DocumentError> {
        let path: &Path = self.path.as_deref().ok_or(DocumentError::NoPath)?;
        if self.write_locked {
            return Err(DocumentError::WriteLocked(path.to_path_buf()));
        }
        let target: PathBuf = resolve(path)?;
        match fs::metadata(&target) {
            Ok(metadata) => self.save_existing(&target, &metadata)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.save_by_rename(&target, None)?;
            }
            Err(error) => return Err(error.into()),
        }
        self.saved_state = self.state;
        Ok(())
    }

    fn save_existing(&self, target: &Path, metadata: &fs::Metadata) -> Result<(), DocumentError> {
        let file: fs::File = fs::OpenOptions::new()
            .write(true)
            .open(target)
            .map_err(|error: std::io::Error| read_only(target, error))?;
        if metadata.nlink() > 1 {
            return self.write_in_place(file);
        }
        self.save_by_rename(target, Some((file, metadata)))
    }

    fn save_by_rename(
        &self,
        target: &Path,
        existing: Option<(fs::File, &fs::Metadata)>,
    ) -> Result<(), DocumentError> {
        let parent: &Path = target.parent().ok_or(DocumentError::NoPath)?;
        let mut temporary: tempfile::NamedTempFile = tempfile::NamedTempFile::new_in(parent)?;
        if let Some((file, metadata)) = existing {
            // fs::copy on macOS copies mode and extended attributes (copyfile).
            fs::copy(target, temporary.path())?;
            let copy: fs::Metadata = temporary.as_file().metadata()?;
            if copy.uid() != metadata.uid() || copy.gid() != metadata.gid() {
                return self.write_in_place(file);
            }
            temporary.as_file_mut().set_len(0)?;
            temporary.as_file_mut().rewind()?;
        }
        self.write_bytes(temporary.as_file_mut())?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(target)
            .map_err(|error: tempfile::PersistError| error.error)?;
        Ok(())
    }

    fn write_in_place(&self, mut file: fs::File) -> Result<(), DocumentError> {
        self.write_bytes(&mut file)?;
        let written: u64 = file.stream_position()?;
        file.set_len(written)?;
        file.sync_all()?;
        Ok(())
    }

    fn write_bytes(&self, out: &mut fs::File) -> Result<(), DocumentError> {
        let mut writer: std::io::BufWriter<&mut fs::File> = std::io::BufWriter::new(out);
        if self.format.bom {
            writer.write_all(&BOM)?;
        }
        self.buffer.write_to(&mut writer)?;
        writer.flush()?;
        Ok(())
    }
}

/// The real file behind `path`: a symlink is followed, a missing file is kept.
fn resolve(path: &Path) -> Result<PathBuf, DocumentError> {
    match fs::canonicalize(path) {
        Ok(real) => Ok(real),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(dangling(path)),
        Err(error) => Err(error.into()),
    }
}

/// A missing file: a dangling symlink points at the file to create.
fn dangling(path: &Path) -> PathBuf {
    match fs::read_link(path) {
        Ok(link) => path
            .parent()
            .map_or(link.clone(), |dir: &Path| dir.join(&link)),
        Err(_) => path.to_path_buf(),
    }
}

fn read_only(target: &Path, error: std::io::Error) -> DocumentError {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        DocumentError::ReadOnly(target.to_path_buf())
    } else {
        DocumentError::Io(error)
    }
}
