use std::cell::RefCell;

use find::{envs::Envs, find_mode::FindMode, temp_file::{FindResult, TempFile}};

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

    let env = Envs::new(&words);
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
    let env = Envs::new(&words);

    let mut file = match TempFile::new() {
        Ok(f) => f,
        Err(err) => {
            assert_eq!(err.to_string(), "".to_string());
            return;
        }
    };

    FindMode::interactive_init(&file, &env);

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
