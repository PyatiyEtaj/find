use std::{fs, io, path::Path};

use crate::regex_helper::RegexHelper;

struct Node {
    current_path: String,
    dirs: Vec<Node>,
    files: Vec<String>
}

pub struct Walker {
    start_point: Option<Node>,
}

impl Walker {
    pub fn new() -> Walker {
        Walker { start_point: None}
    }

    pub fn walk<F: Fn(&String), S: AsRef<str>>(
        &self,
        full_path: S,
        on_find: &F,
        ignore_helper: &RegexHelper,
    ) -> io::Result<()> {
        let read_result = fs::read_dir(full_path.as_ref());

        let dir = match read_result {
            Ok(dir) => dir,
            Err(msg) => {
                println!("[ERR] {:?} err={:?}", full_path.as_ref(), msg);
                return Ok(());
            }
        };

        let ignore = if ignore_helper.is_empty() {
            &RegexHelper::from_gitignore()
        } else {
            ignore_helper
        };

        for info_dir in dir {
            let information = match info_dir {
                Ok(information) => information,
                Err(_) => continue,
            };

            let file_type = match information.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };

            let file_name = match information.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let full_path = &format!("{}/{}", full_path.as_ref(), file_name);

            let ignore_node = ignore.check(full_path);

            if ignore_node {
                continue;
            }

            if file_type.is_file() {
                on_find(full_path);
            } else if file_type.is_dir() {
                self.walk(full_path, on_find, ignore)?;
            }
        }

        Ok(())
    }
}

impl Iterator for Walker {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        

        None
    }
}

#[cfg(test)]
mod walker_tests {
    use super::Walker;

    #[test]
    fn creation() {
        let walker = Walker::new();
    }
}
