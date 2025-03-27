use std::env;

pub struct Envs {
    pub pattern: String,
    pub max_output_lines: i32,
    pub interactive: bool,
    pub start_path: String,
}

impl Envs {
    pub fn new(words: &[String]) -> Envs {
        let mut result = Envs {
            interactive: false,
            max_output_lines: 10,
            pattern: String::new(),
            start_path: env::current_dir().unwrap().to_str().unwrap().to_string().replace(r"\", "/"),
        };

        for i in words.iter().skip(1) {
            if i.starts_with("--line=") {
                if let Some(stripped) = i.strip_prefix("--line=") {
                    result.max_output_lines = stripped.parse::<i32>().unwrap_or(10);
                }
            } else if i.starts_with("-p") {
                if let Some(stripped) = i.strip_prefix("-p=") {
                    result.start_path = stripped.to_string();
                }
            } else {
                result.pattern.push_str(i.as_str());
                result.pattern.push(' ');
            }
        }
        result.pattern = String::from(result.pattern.trim());

        if result.pattern.is_empty() {
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
        assert!(!env.interactive);
        assert_eq!(env.max_output_lines, 11);
    }
}
