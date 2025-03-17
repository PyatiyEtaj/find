use std::{
    fs::{self, File},
    io::{self, Read, Seek, Write},
    sync::{Arc, Mutex},
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

    fn to_string(&self) -> String {
        let mut str = String::new();
        str.push_str(format!("interactive: '{}' | ", self.interactive).as_str());
        str.push_str(format!("line: '{}' | ", self.max_output_lines).as_str());
        str.push_str(format!("pattern: '{}' | ", self.pattern).as_str());
        str.push_str(format!("start-path: '{}'", self.start_path).as_str());

        str
    }
}

struct TempFile {
    name: String,
    f: File,
    seek: u64,
}

#[derive(PartialEq)]
enum FindResult {
    Error(String),
    Read,
    Eof,
}

impl TempFile {
    fn new() -> Result<TempFile, String> {
        let dir = std::env::temp_dir();

        let temp_file_path = &std::path::Path::new(dir.as_os_str()).join("find.txt");

        let f = match std::fs::File::create(temp_file_path) {
            Ok(f) => f,
            Err(_) => return Err("cant create temp file".to_string()),
        };

        Ok(TempFile {
            name: String::from(temp_file_path.to_str().unwrap()),
            f: f,
            seek: 0,
        })
    }

    fn from(file_path: String) -> Result<TempFile, String> {
        let temp_file_path = &std::path::Path::new(file_path.as_str());

        let f = match std::fs::File::open(temp_file_path) {
            Ok(f) => f,
            Err(_) => return Err("cant create temp file".to_string()),
        };

        Ok(TempFile {
            name: String::from(temp_file_path.to_str().unwrap()),
            f: f,
            seek: 0,
        })
    }

    fn refresh(&mut self) {
        self.seek = 0;
    }

    fn find<F: Fn(&String)>(&mut self, pattern: &String, on_find: &F) -> FindResult {
        match self.f.seek(io::SeekFrom::Start(self.seek)) {
            Ok(_) => {}
            Err(err) => return FindResult::Error(err.to_string()),
        };

        const SIZE: usize = 128 * 1024;

        let mut buf = vec![0; SIZE];

        match self.f.read(&mut buf) {
            Ok(read) => {
                self.seek += read as u64;
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
            self.seek -= last.len() as u64;
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
    fn straight(program_envs: Envs) -> io::Result<()> {
        initialize_search(&program_envs.start_path, &mut |node_name, _| {
            if node_name.contains(&program_envs.pattern) {
                println!("{}", node_name);
            };
        })?;

        Ok(())
    }

    fn interactive(program_envs: Envs) -> io::Result<()> {
        let tf = match TempFile::new() {
            Ok(f) => f,
            Err(err) => {
                println!("[ERR] {}", err);
                return Ok(());
            }
        };

        let arc_tf = Arc::new(Mutex::new(tf));

        initialize_search(&program_envs.start_path, &mut |node_name, _| {
            let write_state = arc_tf
                .lock()
                .unwrap()
                .f
                .write_fmt(format_args!("{}\n", node_name));

            match write_state {
                Ok(_) => {}
                Err(err) => println!("[ERR] cant write err={}", err.to_string()),
            }
        })?;

        println!("temp file: {}", arc_tf.lock().unwrap().name);

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let words: Vec<String> = std::env::args().map(|e| e).collect();

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

    use crate::{initialize_search, Envs, FindResult, TempFile};

    fn get_env_1() -> Vec<String> {
        vec![
            r".\projects\file\file\target\release\file.exe".to_string(),
            r"'some pattern .*".to_string(),
            r"--path=.\some\dir".to_string(),
            r"--line=11".to_string(),
        ]
    }

    fn get_env_2() -> Vec<String> {
        vec![
            r".\projects\file\file\target\release\file.exe".to_string(),
            r"main.rs".to_string(),
            r"--path=.".to_string(),
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

    #[test]
    fn temp_file_test() {
        let file_path = "test/find.txt";
        let mut file = match TempFile::from(file_path.to_string()) {
            Ok(f) => f,
            Err(err) => {
                assert_eq!(err.to_string(), "".to_string());
                return;
            }
        };

        let has_been_found = RefCell::new(false);
        let mut is_eof = false;
        while !is_eof {
            let find_result = file.find(&"Cargo.lock".to_string(), &|f| {
                has_been_found.replace(true);
            });

            match find_result {
                FindResult::Error(err) => assert_eq!(err.to_string(), "".to_string()),
                FindResult::Read => {}
                FindResult::Eof => is_eof = true,
                FindResult::None => {}
            }
        }

        assert!(has_been_found.take())
    }
}
