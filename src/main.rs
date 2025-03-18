use std::{
    cell::RefCell,
    fs::{self, File},
    io::{self, Read, Seek, Write},
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
};

struct Envs {
    pattern: String,
    max_output_lines: i32,
    interactive: bool,
    start_path: String,
}

impl Envs {
    fn new(words: &Vec<String>) -> Result<Envs, String> {
        if words.len() < 2 {
            return Err(
                "must be at least 1 arg regex pattern or --interactive [--line]".to_string(),
            );
        }

        let mut result = Envs {
            interactive: false,
            max_output_lines: 10,
            pattern: String::new(),
            start_path: ".".to_string(),
        };

        for i in 1..words.len() {
            if words[i].starts_with("--interactive") {
                result.interactive = true;
            } else if words[i].starts_with("--line=") {
                result.max_output_lines = match words[i]["--line=".len()..].parse::<i32>() {
                    Ok(value) => value,
                    Err(_) => 10,
                };
            } else if words[i].starts_with("--path=") {
                result.start_path = String::from(&words[i]["--path=".len()..]);
            } else {
                result.pattern.push_str(words[i].as_str());
                result.pattern.push_str(" ");
            }
        }
        result.pattern = String::from(result.pattern.trim());

        if result.pattern.len() < 1 && !result.interactive {
            return Err("cant find any pattern".to_string());
        }

        Ok(result)
    }
}

#[derive(PartialEq)]
enum FindResult {
    Error(String),
    Read,
    Eof,
}

struct TempFile {
    name: String,
    write: Option<File>,
    read: File,
    read_seek: u64,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.name);
    }
}

impl TempFile {
    fn new() -> Result<TempFile, String> {
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

    fn from(file_path: String) -> Result<TempFile, String> {
        let temp_file_path = &std::path::Path::new(file_path.as_str());

        let f = match std::fs::File::open(temp_file_path) {
            Ok(f) => f,
            Err(_) => return Err("cant open read-only temp file".to_string()),
        };

        Ok(TempFile {
            name: String::from(temp_file_path.to_str().unwrap()),
            write: None,
            read: f,
            read_seek: 0,
        })
    }

    fn refresh(&mut self) {
        self.read_seek = 0;
    }

    fn find<F: Fn(&String)>(&mut self, pattern: &String, on_find: &F) -> FindResult {
        match self.read.seek(io::SeekFrom::Start(self.read_seek)) {
            Ok(_) => {}
            Err(err) => return FindResult::Error(err.to_string()),
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
            if s.contains(pattern) {
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

fn initialize_search<F: Fn(&String, bool)>(full_path: &String, on_find: &F) -> io::Result<()> {
    let read_result = fs::read_dir(full_path);

    let dir = match read_result {
        Ok(dir) => dir,
        Err(msg) => {
            println!("{:?} err={:?}", full_path, msg);
            return Ok(());
        }
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
        let full_path = &format!("{full_path}/{file_name}");

        if file_type.is_file() {
            on_find(full_path, false);
        } else if file_type.is_dir() {
            on_find(full_path, true);
            initialize_search(full_path, on_find)?;
        }
    }

    Ok(())
}

struct Mode {}

impl Mode {
    pub fn straight(program_envs: Envs) -> io::Result<()> {
        initialize_search(&program_envs.start_path, &mut |node_name, _| {
            if node_name.contains(&program_envs.pattern) {
                println!("{}", node_name);
            };
        })?;

        Ok(())
    }

    pub fn interactive_init(tf: &TempFile, program_envs: &Envs) {
        let to_write = match &tf.write {
            Some(write_f) => write_f,
            None => {
                return;
            }
        };

        let arc_tf = Arc::new(Mutex::new(to_write));

        let _ = initialize_search(&program_envs.start_path, &mut |node_name, _| {
            let write_state = arc_tf
                .lock()
                .unwrap()
                .write_fmt(format_args!("{}\n", node_name));

            match write_state {
                Ok(_) => {}
                Err(err) => println!("[ERR] cant write err={}", err.to_string()),
            }
        });
    }

    fn read_from_stdin() -> Option<String> {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut pattern = String::new();
        std::io::stdin().read_line(&mut pattern).unwrap();
        if pattern.starts_with("q") || pattern.starts_with("quit") || pattern.starts_with("exit") {
            return None;
        }

        Some(pattern.trim().to_string())
    }

    pub fn interactive_find_pattern(tf: &mut TempFile, pattern: &String, program_envs: &Envs) {
        tf.refresh();
        let search = AtomicBool::new(true);
        let found = AtomicI32::new(0);
        while search.load(Ordering::Relaxed) {
            let find_result = tf.find(&pattern, &|f| {
                let prev = found.fetch_add(1, Ordering::Relaxed);
                if program_envs.max_output_lines < 0
                    || found.load(Ordering::Relaxed) <= program_envs.max_output_lines
                {
                    println!("{}) {}", prev + 1, f);
                }
            });

            if found.load(Ordering::Relaxed) >= program_envs.max_output_lines {
                search.store(false, Ordering::Relaxed);
            }

            match find_result {
                FindResult::Error(err) => println!("[ERR] cant read; err={}", err.to_string()),
                FindResult::Read => {}
                FindResult::Eof => {
                    search.store(false, Ordering::Relaxed);
                }
            }
        }

        let val = found.load(Ordering::Relaxed);
        if val < 1 {
            println!("");
        } else {
            println!("found {}", val);
        }
    }

    pub fn interactive(program_envs: Envs) -> io::Result<()> {
        let mut tf = match TempFile::new() {
            Ok(f) => f,
            Err(err) => {
                println!("[ERR] {}", err);
                return Ok(());
            }
        };

        let start = std::time::Instant::now();
        Mode::interactive_init(&tf, &program_envs);
        println!(
            "temp file: {} / took {} ms",
            tf.name,
            start.elapsed().as_millis()
        );

        loop {
            let pattern = match Self::read_from_stdin() {
                Some(p) => p,
                None => break,
            };

            Self::interactive_find_pattern(&mut tf, &pattern, &program_envs);
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let words: Vec<String> = std::env::args().map(|e| e).collect();

    //let words: Vec<String> = vec!["1".to_string(), "--interactive".to_string()];

    let program_envs = match Envs::new(&words) {
        Ok(res) => res,
        Err(err) => {
            println!("[ERR] {}", err);
            return Ok(());
        }
    };

    if program_envs.interactive {
        Mode::interactive(program_envs)?;
    } else {
        Mode::straight(program_envs)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{initialize_search, Envs, FindResult, Mode, TempFile};

    fn get_env_1() -> Vec<String> {
        vec![
            r".\projects\file\file\target\release\file.exe".to_string(),
            r"'some pattern .*".to_string(),
            r"--path=.\some\dir".to_string(),
            r"--line=11".to_string(),
        ]
    }

    #[test]
    fn parsing_envs_test() {
        let words = get_env_1();

        let program_envs_result = Envs::new(&words);

        assert!(program_envs_result.is_ok());
        let env = program_envs_result.unwrap();

        assert_eq!(env.pattern, "'some pattern .*".to_string());
        assert_eq!(env.start_path, r".\some\dir".to_string());
        assert_eq!(env.interactive, false);
        assert_eq!(env.max_output_lines, 11);
    }

    fn get_env_2() -> Vec<String> {
        vec![
            r".\projects\file\file\target\release\file.exe".to_string(),
            r"main.rs".to_string(),
            r"--path=.".to_string(),
        ]
    }

    #[test]
    fn search_pattern_test() {
        let words = get_env_2();

        let program_envs_result = Envs::new(&words);
        assert!(program_envs_result.is_ok());
        let env = program_envs_result.unwrap();
        let has_been_found = RefCell::new(false);

        initialize_search(&env.start_path, &mut |file, is_dir| {
            if file.contains(&env.pattern) {
                assert_eq!(*file, r"./src/main.rs".to_string());
                assert_eq!(is_dir, false);
                has_been_found.replace(true);
            };
        })
        .unwrap();

        assert!(has_been_found.take())
    }

    fn get_env_3() -> Vec<String> {
        vec![
            r".\target\release\file.exe".to_string(),
            r"main.rs".to_string(),
        ]
    }

    #[test]
    fn temp_file_test() {
        let words = get_env_3();
        let program_envs_result = Envs::new(&words);
        assert!(program_envs_result.is_ok());
        
        let mut file = match TempFile::new() {
            Ok(f) => f,
            Err(err) => {
                assert_eq!(err.to_string(), "".to_string());
                return;
            }
        };

        Mode::interactive_init(&file, &program_envs_result.unwrap());

        let has_been_found = RefCell::new(false);
        
        loop {
            let find_result = file.find(&"Cargo.lock".to_string(), &|_| {
                has_been_found.replace(true);
            });

            match find_result {
                FindResult::Error(err) => assert_eq!(err.to_string(), "".to_string()),
                FindResult::Read => {}
                FindResult::Eof => break,
            }
        }

        assert!(has_been_found.take())
    }
}
