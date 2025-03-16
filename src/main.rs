use std::{
    fs::{self, File},
    io,
};

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
        return Err("must started with - or --".to_string());
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

struct TreeInfo {
    name: String,
}

struct TreeDir {
    me: TreeInfo,
    sub_dirs: Vec<Box<TreeDir>>,
    files: Vec<TreeInfo>,
}

impl TreeDir {
    pub fn new(path: String) -> Self {
        TreeDir {
            me: TreeInfo { name: path },
            files: Vec::new(),
            sub_dirs: Vec::new(),
        }
    }

    fn initialize_path(&mut self) -> io::Result<()> {
        self._initialize_path(self.me.name.clone())
    }

    fn _initialize_path(&mut self, full_path: String) -> io::Result<()> {
        dbg!(&full_path);

        let read_result = fs::read_dir(&full_path);

        let dir = match read_result {
            Ok(dir) => dir,
            Err(msg) => {
                print!("{:?}", msg);
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

            if file_type.is_file() {
                self.files.push(TreeInfo { name: file_name });
            } else if file_type.is_dir() {
                let tree = TreeDir::new(file_name);
                self.sub_dirs.push(Box::new(tree));
            }
        }

        for dir in &mut self.sub_dirs {
            let name = dir.me.name.clone();
            dir._initialize_path(format!("{full_path}/{name}"))?;
        }

        Ok(())
    }

    fn _to_string(&self, offset: usize) {
        let mut draw = "".to_string();

        let mut prefix = String::with_capacity(offset);
        for _i in 0..offset {
            prefix.push_str("-");
        }

        draw.push_str(prefix.as_str());
        draw.push_str(">[d] ");
        draw.push_str(self.me.name.as_str());
        draw.push_str("\n");

        for file in &self.files {
            draw.push_str(prefix.as_str());
            draw.push_str("[f] ");
            draw.push_str(file.name.as_str());
            draw.push_str("\n");
        }

        print!("{}", draw);

        draw.clear();

        for dir in &self.sub_dirs {
            dir._to_string(offset + 2);
        }
    }

    pub fn to_string(&self) {
        self._to_string(0);
    }
}

fn find(pe: &Envs, tree: &TreeDir)
{
}

fn main() -> io::Result<()> {
    let words = vec![
        "--path=.".to_string(),
        "--search=libfile-ff7444b4ac3395ea.rmeta".to_string(),
    ];
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

    let mut tree = TreeDir::new(program_envs.path.value);
    tree.initialize_path()?;
    tree.to_string();



    Ok(())
}
