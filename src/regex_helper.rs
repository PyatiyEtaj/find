use regex::Regex;
use std::io::{BufRead, Read};

pub struct RegexHelper {
    name: String,
    regexes: Vec<Regex>,
}

impl RegexHelper {
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

        Ok(RegexHelper {
            name: path,
            regexes: regexes,
        })
    }

    pub fn from_string(pattern: &String) -> Result<RegexHelper, String> {
        let r = match Regex::new(pattern) {
            Ok(r) => r,
            Err(err) => return Err(err.to_string()),
        };

        Ok(RegexHelper {
            name: "".to_string(),
            regexes: vec![r],
        })
    }

    pub fn check(&self, str: &String) -> bool {
        for r in &self.regexes {
            if r.is_match(&str) {
                return true;
            }
        }

        false
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
