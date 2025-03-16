use std::{fs, io, time::Instant};

struct Envs {
    pattern: String,
    max_output_lines: i32,
    interactive: bool,
    start_path: String,
}

impl Envs {
    fn new(words: &Vec<String>) -> Result<Envs, &str> {
        if words.len() < 2 {
            return Err("must be at least 1 arg regex pattern or --interactive [--line]");
        }

        let mut result = Envs {
            interactive: false,
            max_output_lines: -1,
            pattern: String::new(),
            start_path: ".".to_string()
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
            }
            else {
                result.pattern.push_str(words[i].as_str());
                result.pattern.push_str(" ");
            }
        }
        result.pattern = String::from(result.pattern.trim());

        if result.pattern.len() < 1 && !result.interactive {
            return Err("cant find any pattern");
        }

        Ok(result)
    }

    fn to_string(&self) -> String{
        let mut str = String::new();
        str.push_str(format!("interactive: '{}' | ", self.interactive).as_str());
        str.push_str(format!("line: '{}' | ", self.max_output_lines).as_str());
        str.push_str(format!("pattern: '{}' | ", self.pattern).as_str());
        str.push_str(format!("start-path: '{}'", self.start_path).as_str());

        str
    }
}

fn initialize_search<F: Fn(&String, bool)>(
    full_path: &String,
    on_find: &F
) -> io::Result<()> {
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

    initialize_search(
        &program_envs.start_path,
        &|file, is_dir| {
            if file.contains(&program_envs.pattern) {
                println!("{}", file);
            };
        }
    )?;

    let init_end_time = start_time.elapsed();

    println!("took {} ms", init_end_time.as_millis());

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{initialize_search, Envs};

    fn get_env_1() -> Vec<String>{
        vec![
            r".\projects\file\file\target\release\file.exe".to_string(),
            r"'some pattern .*".to_string(),
            r"--path=.\some\dir".to_string(),
            r"--line=10".to_string(),
        ]
    }
    
    fn get_env_2() -> Vec<String>{
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
        let env = program_envs_result .unwrap();

        assert_eq!(env.pattern, "'some pattern .*".to_string());
        assert_eq!(env.start_path, r".\some\dir".to_string());
        assert_eq!(env.interactive, false);
        assert_eq!(env.max_output_lines, 10);
    }

    #[test]
    fn search_pattern_test(){
        let words = get_env_2();

        let program_envs_result = Envs::new(&words);
        assert!(program_envs_result.is_ok());
        let env = program_envs_result .unwrap();

        initialize_search(
            &env.start_path,
            &|file, is_dir| {
                if file.contains(&env.pattern) {
                    assert_eq!(*file, r"./src/main.rs".to_string());
                    assert_eq!(is_dir, false);
                };
            }
        ).unwrap();
    }
}