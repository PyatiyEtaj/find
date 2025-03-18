pub struct Envs {
    pub pattern: String,
    pub max_output_lines: i32,
    pub interactive: bool,
    pub start_path: String,
}

impl Envs {
    pub fn new(words: &Vec<String>) -> Envs {
        let mut result = Envs {
            interactive: false,
            max_output_lines: 10,
            pattern: String::new(),
            start_path: ".".to_string(),
        };

        for i in 1..words.len() {
            if words[i].starts_with("--line=") {
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

        if result.pattern.len() < 1 {
            result.interactive = true;
        }

        result
    }
}

#[cfg(test)]
mod envs_tests {
    use crate::envs::Envs;

    fn get_env_1() -> Vec<String> {
        vec![
            r".\projects\file\file\target\release\file.exe".to_string(),
            r"'some pattern .*".to_string(),
            r"--path=.\some\dir".to_string(),
            r"--line=11".to_string(),
        ]
    }

    #[test]
    fn parsing_envs() {
        let words = get_env_1();

        let env = Envs::new(&words);

        assert_eq!(env.pattern, "'some pattern .*".to_string());
        assert_eq!(env.start_path, r".\some\dir".to_string());
        assert_eq!(env.interactive, false);
        assert_eq!(env.max_output_lines, 11);
    }
}
