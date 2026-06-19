use std::{
    env,
    f32::consts::PI,
    fs::File,
    io::{self, BufWriter, Write},
    sync::{Arc, Mutex},
    thread,
};

use crossterm::{
    event::{read, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::{envs::Envs, regex_helper::RegexHelper, temp_file, walker::Walker};

use temp_file::TempFile;

pub struct FindMode {}

impl FindMode {
    pub fn straight(program_envs: Envs) -> io::Result<()> {
        let s = match RegexHelper::from_string(&program_envs.pattern) {
            Ok(s) => s,
            Err(err) => {
                eprintln!("[ERR] err={}", err);
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

    fn interactive_init(tf: BufWriter<File>, program_envs: &Envs) {
        let arc_tf = Arc::new(Mutex::new(tf));

        let (send, receive) = flume::unbounded::<String>();

        let path = program_envs.start_path.clone();

        const END_TOK: &str = ":END";

        let walker_th = thread::spawn(move || {
            _ = Walker::walk_by_channel(&send, path, &RegexHelper::default());
            _ = send.send(END_TOK.to_string());
        });

        let writer_th = thread::spawn(move || {
            while let Ok(path) = &receive.recv() {
                if path == END_TOK {
                    println!("[FS HAS BEEN SCANNED]");
                    break;
                }

                let write_state = arc_tf.lock().unwrap().write_fmt(format_args!("{}\n", path));

                if let Err(e) = write_state {
                    eprintln!("[ERR] cant write err={}", e);
                }
            }
            _ = arc_tf.lock().unwrap().flush();
        });
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

        let searcher = match RegexHelper::from_string(pattern) {
            Ok(s) => s,
            Err(err) => {
                eprintln!("[ERR] {}", err);
                return;
            }
        };

        let buf = tf.as_raw().unwrap();
        let mut count = 0;
        let mut at = 0;
        let mut done = false;
        while !done && (program_envs.max_output_lines < 0 || count <= program_envs.max_output_lines)
        {
            let line = match memchr::memchr(b'\n', &buf[at..]) {
                Some(p) => {
                    let line = &buf[at..at + p];
                    at += p + 1;
                    line
                }
                None => {
                    done = true;
                    let line = &buf[at..];
                    line
                }
            };

            let s = unsafe { str::from_utf8_unchecked(line) };
            if searcher.check(s) {
                count += 1;
                println!("{}) {}", count, s);
            }
        }

        if program_envs.max_output_lines > 0 && count >= program_envs.max_output_lines {
            println!("... some more\n");
        } else {
            println!();
        }
    }

    pub fn interactive(program_envs: Envs) -> io::Result<()> {
        let mut tf = match TempFile::new() {
            Ok(f) => f,
            Err(err) => {
                eprintln!("[ERR] {}", err);
                return Ok(());
            }
        };

        let f = tf.write.take().unwrap();
        let bw = BufWriter::new(f);
        FindMode::interactive_init(bw, &program_envs);
        println!("temp file: {} / press Esc to exit", tf.name);

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
                    Err(err) => eprintln!("[ERR] cant write err={}", err),
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
                eprintln!("[ERR] {}", err);
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
