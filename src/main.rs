mod envs;
mod find_mode;
mod temp_file;

use std::io;

use envs::Envs;
use find_mode::FindMode;

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
        FindMode::interactive(program_envs)?;
    } else {
        FindMode::straight(program_envs)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{
        envs::Envs,
        find_mode::FindMode,
        temp_file::{FindResult, TempFile},
    };

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

        FindMode::initialize_search(&env.start_path, &mut |file, is_dir| {
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

        FindMode::interactive_init(&file, &program_envs_result.unwrap());

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
