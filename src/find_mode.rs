use std::{
    fs,
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
};

use crate::{envs::Envs, regex_helper::RegexHelper, temp_file};

use temp_file::{FindResult, TempFile};

pub struct FindMode {}

impl FindMode {
    pub fn initialize_search<F: Fn(&String, bool)>(
        full_path: &String,
        on_find: &F,
    ) -> io::Result<()> {
        let read_result = fs::read_dir(full_path);

        let dir = match read_result {
            Ok(dir) => dir,
            Err(msg) => {
                println!("[ERR] {:?} err={:?}", full_path, msg);
                return Ok(());
            }
        };

        let ignore = match RegexHelper::from_file(".gitignore".to_string()){
            Ok(mut s) => {
                s.add_pattern(&"/.git".to_string()).unwrap();
                Some(s)
            },
            Err(_) => None,
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

            let ignore_node = match ignore {
                Some(ref s) => s.check(full_path),
                None => false,
            };

            if ignore_node{
                continue;
            }

            if file_type.is_file() {
                on_find(full_path, false);
            } else if file_type.is_dir() {
                on_find(full_path, true);
                Self::initialize_search(full_path, on_find)?;
            }
        }

        Ok(())
    }

    pub fn straight(program_envs: Envs) -> io::Result<()> {
        let s = match RegexHelper::from_string(&program_envs.pattern) {
            Ok(s) => s,
            Err(err) => {
                println!("[ERR] err={}", err);
                return Ok(());
            }
        };

        Self::initialize_search(&program_envs.start_path, &mut |node_name, _| {
            if s.check(node_name) {
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

        let _ = Self::initialize_search(&program_envs.start_path, &mut |node_name, _| {
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
        FindMode::interactive_init(&tf, &program_envs);
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
