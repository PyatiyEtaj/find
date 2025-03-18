use std::{
    fs::File,
    io::{self, Read, Seek},
};

use crate::regex_helper::RegexHelper;

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

        let temp_file_path = &std::path::Path::new(dir.as_os_str()).join("find.txt");

        let to_write = match std::fs::File::create(temp_file_path) {
            Ok(f) => f,
            Err(_) => return Err("cant create write-only temp file".to_string()),
        };

        let to_read = match std::fs::File::open(temp_file_path) {
            Ok(f) => f,
            Err(_) => return Err("cant open temp file as read-only".to_string()),
        };

        Ok(TempFile {
            name: String::from(temp_file_path.to_str().unwrap()),
            write: Some(to_write),
            read: to_read,
            read_seek: 0,
        })
    }

    pub fn refresh(&mut self) {
        self.read_seek = 0;
    }

    pub fn find<F: Fn(&String)>(&mut self, pattern: &String, on_find: &F) -> FindResult {
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
            if searcher.check_str(s) {
                on_find(&s.to_string());
            }
            last = s;
        }

        if last.len() > 0 && !last.ends_with('\0') && !last.ends_with('\n') {
            self.read_seek -= last.len() as u64;
        }

        FindResult::Read
    }
}
