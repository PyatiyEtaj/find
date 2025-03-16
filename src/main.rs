use std::{
    fs::{self, File},
    io::{self, BufRead, Read, Seek, Write},
    str::FromStr,
    string,
    sync::{Arc, Mutex},
    time::Instant,
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
            max_output_lines: -1,
            pattern: String::new(),
            start_path: ".".to_string(),
        };

        for i in 1..words.len() {
            if words[i].starts_with("--interactive") {
                result.interactive = true;
            } else if words[i].starts_with("--line=") {
                result.max_output_lines = match words[i]["--line=".len()..].parse::<i32>() {
                    Ok(value) => value,
                    Err(_) => -1,
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
}

enum FindResult {
    Ok(String),
    Error(String),
    None,
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
        })
    }

    fn find(&mut self, pattern: &String, seek_from_start: Option<bool>) -> FindResult {
        let seek = match seek_from_start {
            Some(v) => {
                if v {
                    io::SeekFrom::Start(0)
                } else {
                    io::SeekFrom::Current(0)
                }
            }
            None => io::SeekFrom::Current(0),
        };

        match self.f.seek(seek) {
            Ok(_) => {}
            Err(err) => return FindResult::Error(err.to_string()),
        };

        const SIZE: usize = 128 * 1024;

        let mut buf = vec![0; SIZE];

        let read = match self.f.read(&mut buf) {
            Ok(read) => read,
            Err(err) => {
                return FindResult::Error(err.to_string());
            }
        };

        let str = match String::from_utf8(buf) {
            Ok(str) => str,
            Err(err) => {
                return FindResult::Error(err.to_string())
            }
        };

        

        //reader.lines().take(n)

        FindResult::None
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

fn main() -> io::Result<()> {
    let words: Vec<String> = std::env::args().map(|e| e).collect();

    let start_time = Instant::now();

    let program_envs = match Envs::new(&words) {
        Ok(res) => res,
        Err(err) => {
            println!("[ERR] {}", err);
            return Ok(());
        }
    };

    println!("envs: {:?}", program_envs.to_string());

    let tf = match TempFile::new() {
        Ok(f) => f,
        Err(err) => {
            println!("[ERR] {}", err);
            return Ok(());
        }
    };

    let arc_tf = Arc::new(Mutex::new(tf));

    initialize_search(&program_envs.start_path, &mut |node_name, is_dir| {
        arc_tf
            .lock()
            .unwrap()
            .f
            .write_fmt(format_args!("{}\n", node_name))
            .unwrap();
        if node_name.contains(&program_envs.pattern) {
            println!("{}", node_name);
        };
    })?;

    let init_end_time = start_time.elapsed();

    println!(
        "took {} ms | temp {}",
        init_end_time.as_millis(),
        arc_tf.lock().unwrap().name
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{initialize_search, Envs, TempFile};

    fn get_env_1() -> Vec<String> {
        vec![
            r".\projects\file\file\target\release\file.exe".to_string(),
            r"'some pattern .*".to_string(),
            r"--path=.\some\dir".to_string(),
            r"--line=10".to_string(),
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
        assert_eq!(env.max_output_lines, 10);
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
        let file = TempFile::new();
    }
}
