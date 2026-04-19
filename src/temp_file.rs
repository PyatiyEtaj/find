use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::regex_helper::RegexHelper;

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[cfg(windows)]
const FILE_SHARE_READ: u32 = 0x0000_0001;
#[cfg(windows)]
const FILE_SHARE_WRITE: u32 = 0x0000_0002;
#[cfg(windows)]
const FILE_SHARE_DELETE: u32 = 0x0000_0004;
#[cfg(windows)]
const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;

#[derive(PartialEq)]
pub enum FindResult {
    Error(String),
    Read,
    Eof,
}

pub struct TempFile {
    pub name: String,
    pub write: Option<File>,
    read: File,
    read_seek: u64,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.name);
    }
}

impl TempFile {
    pub fn new() -> Result<TempFile, String> {
        let dir = std::env::temp_dir();
        let temp_file_path = Self::create_unique_temp_file_path(&dir);

        let to_write = match Self::create_temp_write_file(&temp_file_path) {
            Ok(f) => f,
            Err(_) => return Err("cant create write-only temp file".to_string()),
        };

        let to_read = match Self::open_temp_read_file(&temp_file_path) {
            Ok(f) => f,
            Err(_) => return Err("cant open temp file as read-only".to_string()),
        };

        Ok(TempFile {
            name: temp_file_path.to_string_lossy().into_owned(),
            write: Some(to_write),
            read: to_read,
            read_seek: 0,
        })
    }

    fn create_unique_temp_file_path(dir: &Path) -> PathBuf {
        let pid = std::process::id();

        loop {
            let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let file_name = format!("rfind-{}-{}-{}.tmp", pid, now, counter);
            let file_path = dir.join(file_name);

            if !file_path.exists() {
                return file_path;
            }
        }
    }

    #[cfg(windows)]
    fn create_temp_write_file(path: &Path) -> io::Result<File> {
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_DELETE_ON_CLOSE)
            .open(path)
    }

    #[cfg(not(windows))]
    fn create_temp_write_file(path: &Path) -> io::Result<File> {
        OpenOptions::new().create_new(true).write(true).open(path)
    }

    #[cfg(windows)]
    fn open_temp_read_file(path: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .open(path)
    }

    #[cfg(not(windows))]
    fn open_temp_read_file(path: &Path) -> io::Result<File> {
        OpenOptions::new().read(true).open(path)
    }

    pub fn refresh(&mut self) {
        self.read_seek = 0;
    }

    pub fn find<F: Fn(&String), S: AsRef<str>>(&mut self, pattern: S, on_find: &F) -> FindResult {
        match self.read.seek(io::SeekFrom::Start(self.read_seek)) {
            Ok(_) => {}
            Err(err) => return FindResult::Error(err.to_string()),
        };

        let searcher = match RegexHelper::from_string(pattern) {
            Ok(s) => s,
            Err(err) => return FindResult::Error(err),
        };

        const SIZE: usize = 128 * 1024;

        let mut buf = vec![0; SIZE];

        match self.read.read(&mut buf) {
            Ok(read) => {
                self.read_seek += read as u64;
                if read < 1 {
                    return FindResult::Eof;
                }
            }
            Err(err) => {
                return FindResult::Error(err.to_string());
            }
        };

        let str = match String::from_utf8(buf) {
            Ok(str) => str,
            Err(err) => {
                return FindResult::Error(err.to_string());
            }
        };

        let splitted = str.split("\n");
        let mut last: &str = "";
        for s in splitted {
            if searcher.check(s) {
                on_find(&s.to_string());
            }
            last = s;
        }

        if !last.is_empty() && !last.ends_with('\0') && !last.ends_with('\n') {
            self.read_seek -= last.len() as u64;
        }

        FindResult::Read
    }
}

#[cfg(test)]
mod temp_file_tests {
    use super::TempFile;

    #[test]
    fn temp_files_use_unique_names() {
        let first = TempFile::new().unwrap();
        let second = TempFile::new().unwrap();

        assert_ne!(first.name, second.name);
    }

    #[test]
    fn temp_file_is_removed_after_drop() {
        let file_name = {
            let temp_file = TempFile::new().unwrap();
            let file_name = temp_file.name.clone();
            assert!(std::path::Path::new(&file_name).exists());
            file_name
        };

        assert!(!std::path::Path::new(&file_name).exists());
    }
}
