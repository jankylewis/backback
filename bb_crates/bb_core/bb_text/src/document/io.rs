use std::{fs, io::Write, path::Path};

use super::{Document, DocumentError};

impl Document {
    /// Open a UTF-8 file, preserving its BOM and line breaks.
    ///
    /// # Errors
    /// Returns a file or UTF-8 error.
    pub fn open(path: &Path) -> Result<Self, DocumentError> {
        let bytes: Vec<u8> = fs::read(path)?;
        let bom: bool = bytes.starts_with(&[0xef, 0xbb, 0xbf]);
        let text: String = String::from_utf8(if bom {
            bytes.get(3..).unwrap_or(&[]).to_vec()
        } else {
            bytes
        })?;
        let mut doc: Self = Self::from_text(&text, Some(path.to_path_buf()));
        doc.bom = bom;
        Ok(doc)
    }

    /// Write the complete file through a same-directory temporary file and rename.
    ///
    /// # Errors
    /// Returns a file error or [`DocumentError::NoPath`].
    pub fn save(&mut self) -> Result<(), DocumentError> {
        let path: &Path = self.path.as_deref().ok_or(DocumentError::NoPath)?;
        let parent: &Path = path.parent().ok_or(DocumentError::NoPath)?;
        let mut temporary: tempfile::NamedTempFile = tempfile::NamedTempFile::new_in(parent)?;
        if let Ok(metadata) = fs::metadata(path) {
            temporary
                .as_file()
                .set_permissions(metadata.permissions())?;
        }
        if self.bom {
            temporary.write_all(&[0xef, 0xbb, 0xbf])?;
        }
        temporary.write_all(self.buffer.to_text().as_bytes())?;
        temporary.flush()?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(path)
            .map_err(|error: tempfile::PersistError| error.error)?;
        self.saved = self.buffer.clone();
        self.dirty = false;
        Ok(())
    }
}
