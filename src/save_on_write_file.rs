use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub struct SaveOnWriteFile {
    file: Option<File>,
    path: PathBuf,
}

impl SaveOnWriteFile {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            file: None,
            path: path.as_ref().to_owned(),
        }
    }

    fn ensure_exists(&mut self) -> std::io::Result<&mut File> {
        if self.file.is_none() {
            let file = File::create(&self.path)?;
            self.file = Some(file);
            return Ok(self.file.as_mut().unwrap());
        }
        if let Some(file) = self.file.as_mut() {
            return Ok(file);
        }
        unreachable!()
    }
}

impl std::io::Write for SaveOnWriteFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let file = self.ensure_exists()?;
        file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(file) = self.file.as_mut() {
            file.flush()?;
        }
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        let file = self.ensure_exists()?;
        file.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        let file = self.ensure_exists()?;
        file.write_fmt(fmt)
    }
}
