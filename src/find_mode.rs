use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
};

use crate::{envs::Envs, regex_helper::RegexHelper, temp_file, walker::Walker};

use temp_file::{FindResult, TempFile};

pub struct FindMode {}

impl FindMode {
    pub fn straight(program_envs: Envs) -> io::Result<()> {
        let s = match RegexHelper::from_string(&program_envs.pattern) {
            Ok(s) => s,
            Err(err) => {
                println!("[ERR] err={}", err);
                return Ok(());
            }
        };

        let ignore = RegexHelper::new();

        let walker = Walker::new();

        walker.walk(
            &program_envs.start_path,
            &|node_name| {
                if s.check(node_name) {
                    println!("{}", node_name);
                };
            },
            &ignore,
        )?;

        Ok(())
    }

    pub fn interactive_init(tf: &TempFile, program_envs: &Envs) {
        let to_write = match &tf.write {
            Some(write_f) => write_f,
            None => {
                return;
            }
        };

        let ignore = RegexHelper::new();

        let arc_tf = Arc::new(Mutex::new(to_write));

        let walker = Walker::new();

        let _ = walker.walk(
            &program_envs.start_path,
            &|node_name| {
                let write_state = arc_tf
                    .lock()
                    .unwrap()
                    .write_fmt(format_args!("{}\n", node_name));

                match write_state {
                    Ok(_) => {}
                    Err(err) => println!("[ERR] cant write err={}", err.to_string()),
                }
            },
            &ignore,
        );
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
