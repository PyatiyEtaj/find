use std::cell::RefCell;

use rfind::{
    envs::Envs,
    find_mode::FindMode,
    regex_helper::RegexHelper,
    temp_file::{FindResult, TempFile},
    walker::Walker,
};

fn get_env_2() -> Vec<String> {
    vec![
        r".\projects\file\file\target\release\file.exe".to_string(),
        r"main.rs".to_string(),
        r"--path=.".to_string(),
    ]
}

#[test]
fn search_pattern() {
    let words = get_env_2();

    let env = Envs::new(&words);
    let has_been_found = RefCell::new(false);
    let checker = RegexHelper::from_string(&env.pattern).unwrap();
    let ignore = RegexHelper::default();

    Walker::walk(
            env.start_path,
            &|file| {
                if checker.check(file) {
                    assert_eq!(*file, r"./src/main.rs".to_string());
                    has_been_found.replace(true);
                };
            },
            &ignore,
        )
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
fn temp_file_find() {
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
        let find_result = file.find("Cargo.lock", &|_| {
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
