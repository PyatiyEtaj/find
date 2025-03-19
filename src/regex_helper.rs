use regex::Regex;
use std::io::BufRead;

pub struct RegexHelper {
    regexes: Vec<Regex>,
}

impl RegexHelper {
    pub fn new() -> RegexHelper {
        RegexHelper { regexes: vec![] }
    }

    pub fn from_file(path: String) -> Result<RegexHelper, String> {
        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(err) => return Err(err.to_string()),
        };

        let lines = std::io::BufReader::new(file)
            .lines()
            .map_while(Result::ok)
            .collect::<Vec<String>>();

        let regexes = lines
            .iter()
            .filter_map(|p| regex::Regex::new(p).ok())
            .collect::<Vec<Regex>>();

        Ok(RegexHelper { regexes: regexes })
    }

    pub fn from_string(pattern: &String) -> Result<RegexHelper, String> {
        let r = match Regex::new(pattern) {
            Ok(r) => r,
            Err(err) => return Err(err.to_string()),
        };

        Ok(RegexHelper { regexes: vec![r] })
    }

    pub fn from_gitignore() -> RegexHelper {
        let path = ".gitignore";

        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(_) => return RegexHelper::new(),
        };

        let lines = std::io::BufReader::new(file)
            .lines()
            .map_while(Result::ok)
            .collect::<Vec<String>>();

        let mut regexes = lines
            .iter()
            .map(|s| {
                s.replace("**", "")
                    .replace("*", "")
                    .replace("/", "")
                    .replace("\\", "")
                    .replace(".", "\\.")
            })
            .filter_map(|p| regex::Regex::new(&p).ok())
            .collect::<Vec<Regex>>();

        regexes.push(Regex::new(".git").unwrap());

        RegexHelper { regexes: regexes }
    }

    pub fn check(&self, str: &String) -> bool {
        for r in &self.regexes {
            if r.is_match(str) {
                return true;
            }
        }

        false
    }

    pub fn check_str(&self, str: &str) -> bool {
        for r in &self.regexes {
            if r.is_match(str) {
                return true;
            }
        }

        false
    }

    pub fn is_empty(&self) -> bool {
        self.regexes.is_empty()
    }
}

#[cfg(test)]
mod ignore_files_tests {
    use crate::regex_helper::RegexHelper;

    #[test]
    fn check_gitignore() {
        let ignore = match RegexHelper::from_file(".gitignore".to_string()) {
            Ok(i) => i,
            Err(err) => {
                assert_eq!(err, "".to_string());
                return;
            }
        };

        assert!(ignore.check(&"haha/target".to_string()));
        assert!(ignore.check(&"hihi/target".to_string()));
        assert!(ignore.check(&"/local_data".to_string()));
        assert!(ignore.check(&"123/local_data".to_string()));
        assert!(ignore.check(&"123/local_data/1234".to_string()));
    }

    #[test]
    fn check_from_string() {
        let ignore = match RegexHelper::from_string(&".*some".to_string()) {
            Ok(i) => i,
            Err(err) => {
                assert_eq!(err, "".to_string());
                return;
            }
        };

        assert!(ignore.check(&"haha/some".to_string()));
        assert!(ignore.check(&"asdgoasogaosomesome".to_string()));
        assert!(!ignore.check(&"soahasme".to_string()));
    }
}
