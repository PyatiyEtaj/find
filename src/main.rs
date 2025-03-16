use std::{fs, io, time::Instant};

#[derive(PartialEq)]
enum NodeType {
    Dir,
    File,
    Any,
}

enum EnvType {
    Path,
    Search(NodeType),
    None,
}

struct EnvArg {
    name: String,
    value: String,
    env_type: EnvType,
}

fn new_env_arg(string: &String) -> Result<EnvArg, String> {
    if !string.starts_with("-") && !string.starts_with("--") {
        return Ok(EnvArg {
            env_type: EnvType::None,
            name: "".to_string(),
            value: "".to_string(),
        });
    }

    let pos_of = match string.find("=") {
        None => return Err("must contain =".to_string()),
        Some(x) => x,
    };

    let (env_name, value) = string.split_at(pos_of);

    let mut arg = EnvArg {
        name: env_name.to_string(),
        value: value[1..].to_string(),
        env_type: EnvType::None,
    };

    arg.resolve_type();

    Ok(arg)
}

impl EnvArg {
    pub fn resolve_type(self: &mut Self) -> bool {
        if self.name.starts_with("--path") || self.name.starts_with("-p") {
            self.env_type = EnvType::Path;
            return true;
        } else if self.name.starts_with("--search") || self.name.starts_with("-s") {
            let t = if self.value.contains(".") {
                NodeType::File
            } else {
                NodeType::Any
            };
            self.env_type = EnvType::Search(t);
            return true;
        }

        false
    }

    pub fn new() -> Self {
        EnvArg {
            env_type: EnvType::None,
            name: "".to_string(),
            value: "".to_string(),
        }
    }
}

struct Envs {
    path: EnvArg,
    search: EnvArg,
}

impl Envs {
    pub fn init(self: &mut Self, arg: EnvArg) {
        match arg.env_type {
            EnvType::Path => self.path = arg,
            EnvType::Search(_) => self.search = arg,
            EnvType::None => {}
        }
    }
}

fn initialize_search<F1: Fn(&String), F2: Fn(&String)>(
    full_path: &String,
    on_file: &F1,
    on_dir: &F2,
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
            on_file(full_path);
        } else if file_type.is_dir() {
            on_dir(full_path);
            initialize_search(full_path, on_file, on_dir)?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let words: Vec<String> = std::env::args().map(|e| e).collect();

    // let words = vec![
    //     "--path=.".to_string(),
    //     "--search=libfile-ff7444b4ac3395ea.rmeta".to_string(),
    // ];

    let start_time = Instant::now();

    let mut program_envs = Envs {
        path: EnvArg::new(),
        search: EnvArg::new(),
    };

    for word in words {
        let res = new_env_arg(&word);

        match res {
            Ok(mut v) => {
                if v.resolve_type() {
                    program_envs.init(v);
                }
            }
            Err(e) => {
                println!("error parsing header '{word}': {e}");
                return Ok(());
            }
        }
    }

    println!(
        "path={} search={}",
        program_envs.path.value, program_envs.search.value
    );

    initialize_search(
        &program_envs.path.value,
        &|file| {
            if file.contains(&program_envs.search.value) {
                println!("{:?}", file);
            };
        },
        &|dir| {
            if dir.contains(&program_envs.search.value) {
                println!("{:?}", dir);
            };
        },
    )?;
    //tree.to_string();
    let init_end_time = start_time.elapsed();
    println!("-- inited took {} ms --", init_end_time.as_millis());

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(())
}
