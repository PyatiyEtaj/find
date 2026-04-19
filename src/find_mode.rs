use std::{
    io::{self, BufWriter, Write},
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
};

use crossterm::{
    event::{read, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
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

        let ignore = RegexHelper::default();

        Walker::walk(
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

        let ignore = RegexHelper::default();

        let arc_tf = Arc::new(Mutex::new(BufWriter::new(to_write)));

        let _ = Walker::walk(
            &program_envs.start_path,
            &|node_name| {
                let write_state = arc_tf
                    .lock()
                    .unwrap()
                    .write_fmt(format_args!("{}\n", node_name));

                match write_state {
                    Ok(_) => {}
                    Err(err) => println!("[ERR] cant write err={}", err),
                }
            },
            &ignore,
        );

        _ = arc_tf.lock().unwrap().flush();
    }

    fn read_from_stdin() -> Option<String> {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut pattern = String::new();

        enable_raw_mode().unwrap();

        loop {
            match read() {
                Ok(Event::Key(key_event)) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Esc => {
                            disable_raw_mode().unwrap();
                            println!();
                            return None;
                        }
                        KeyCode::Enter => {
                            disable_raw_mode().unwrap();
                            println!();
                            return Some(pattern.trim().to_string());
                        }
                        KeyCode::Backspace => {
                            if pattern.pop().is_some() {
                                print!("\u{8} \u{8}");
                                std::io::stdout().flush().unwrap();
                            }
                        }
                        KeyCode::Char(c) => {
                            pattern.push(c);
                            print!("{}", c);
                            std::io::stdout().flush().unwrap();
                        }
                        _ => {}
                    }
                }
                Ok(_) => {}
                Err(_) => {
                    disable_raw_mode().unwrap();
                    println!();
                    return None;
                }
            }
        }
    }

    pub fn interactive_find_pattern(tf: &mut TempFile, pattern: &String, program_envs: &Envs) {
        tf.refresh();
        let search = AtomicBool::new(true);
        let found = AtomicI32::new(0);
        while search.load(Ordering::Relaxed) {
            let find_result = tf.find(pattern, &|f| {
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
                FindResult::Error(err) => {
                    println!("[ERR] {}", err);
                    search.store(false, Ordering::Relaxed);
                }
                FindResult::Read => {}
                FindResult::Eof => {
                    search.store(false, Ordering::Relaxed);
                }
            }
        }

        if program_envs.max_output_lines > 0 && found.load(Ordering::Relaxed) >= program_envs.max_output_lines {
            println!("... some more\n");
        } else {
            println!();
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
            "temp file: {} / took {} ms / press Esc to exit",
            tf.name,
            start.elapsed().as_millis()
        );

        while let Some(pattern) = Self::read_from_stdin() {
            Self::interactive_find_pattern(&mut tf, &pattern, &program_envs);
        }

        Ok(())
    }
}

impl FindMode {
    pub async fn interactive_init_async(tf: &TempFile, program_envs: &Envs) {
        let to_write = match &tf.write {
            Some(write_f) => write_f,
            None => {
                return;
            }
        };

        let ignore = RegexHelper::default();

        let arc_tf = Arc::new(Mutex::new(BufWriter::new(to_write)));

        let _ = Walker::walk_async(
            &program_envs.start_path,
            &|node_name| {
                let write_state = arc_tf
                    .lock()
                    .unwrap()
                    .write_fmt(format_args!("{}\n", node_name));

                match write_state {
                    Ok(_) => {}
                    Err(err) => println!("[ERR] cant write err={}", err),
                }
            },
            &ignore,
        )
        .await;
    }

    pub async fn interactive_async(program_envs: Envs) -> io::Result<()> {
        let mut tf = match TempFile::new() {
            Ok(f) => f,
            Err(err) => {
                println!("[ERR] {}", err);
                return Ok(());
            }
        };

        let start = std::time::Instant::now();

        FindMode::interactive_init_async(&tf, &program_envs).await;

        println!(
            "temp file: {} / took {} ms / press Esc to exit",
            tf.name,
            start.elapsed().as_millis()
        );

        while let Some(pattern) = Self::read_from_stdin() {
            Self::interactive_find_pattern(&mut tf, &pattern, &program_envs);
        }

        Ok(())
    }
}
