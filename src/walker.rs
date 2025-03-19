use std::{fs, io};

use crate::regex_helper::RegexHelper;

pub struct Walker {}

impl Walker {
    pub fn new() -> Walker {
        Walker {}
    }

    pub fn walk<F: Fn(&String), S: AsRef<str>>(
        &self,
        full_path: S,
        on_file: &F,
        ignore: &RegexHelper,
    ) -> io::Result<()> {
        let read_result = fs::read_dir(full_path.as_ref());

        let dir = match read_result {
            Ok(dir) => dir,
            Err(msg) => {
                println!("[ERR] {:?} err={:?}", full_path.as_ref(), msg);
                return Ok(());
            }
        };

        let ignore = if ignore.is_empty() {
            &RegexHelper::from_gitignore(&full_path)
        } else {
            ignore
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
                on_file(full_path);
            } else if file_type.is_dir() {
                self.walk(full_path, on_file, ignore)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod walker_tests {
    use std::cell::RefCell;

    use crate::regex_helper::RegexHelper;

    use super::Walker;

    #[test]
    fn simple_walk() {
        let walker = Walker::new();
        let ignore = RegexHelper::new();
        let search = RegexHelper::from_string("main.rs").unwrap();
        let has_been_found = RefCell::new(false);
        _ = walker.walk(
            "..",
            &|name| {
                if search.check(name) {
                    has_been_found.replace(true);
                }
            },
            &ignore,
        );

        assert!(has_been_found.take());
    }
}
