pub struct Envs {
    pub pattern: String,
    pub max_output_lines: i32,
    pub interactive: bool,
    pub start_path: String,
}

impl Envs {
    pub fn new(words: &Vec<String>) -> Result<Envs, String> {
        if words.len() < 2 {
            return Err(
                "must be at least 1 arg regex pattern or --interactive [--line]".to_string(),
            );
        }

        let mut result = Envs {
            interactive: false,
            max_output_lines: 10,
            pattern: String::new(),
            start_path: ".".to_string(),
        };

        for i in 1..words.len() {
            if words[i].starts_with("--interactive") {
                result.interactive = true;
            } else if words[i].starts_with("--line=") {
                result.max_output_lines = match words[i]["--line=".len()..].parse::<i32>() {
                    Ok(value) => value,
                    Err(_) => 10,
                };
            } else if words[i].starts_with("--path=") {
                result.start_path = String::from(&words[i]["--path=".len()..]);
            } else {
                result.pattern.push_str(words[i].as_str());
                result.pattern.push_str(" ");
            }
        }
        result.pattern = String::from(result.pattern.trim());

        if result.pattern.len() < 1 && !result.interactive {
            return Err("cant find any pattern".to_string());
        }

        Ok(result)
    }
}