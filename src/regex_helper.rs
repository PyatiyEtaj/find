use regex::Regex;
use std::io::BufRead;

#[derive(Default)]
pub struct RegexHelper {
    regexes: Vec<Regex>,
}

impl RegexHelper {
    pub fn from_string<S: AsRef<str>>(pattern: S) -> Result<RegexHelper, String> {
        let r = match Regex::new(pattern.as_ref()) {
            Ok(r) => r,
            Err(err) => return Err(err.to_string()),
        };

        Ok(RegexHelper { regexes: vec![r] })
    }

    pub fn from_gitignore<P: AsRef<str>>(dir: P) -> RegexHelper {
        let path = std::path::Path::new(dir.as_ref()).join(".gitignore");

        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(_) => return RegexHelper::default(),
        };

        let lines = std::io::BufReader::new(file)
            .lines()
            .map_while(Result::ok)
            .collect::<Vec<String>>();

        let mut regexes = lines
            .iter()
            .map(|s| {
                s.replace("**", "$$$")
                    .replace("*", "[^/]*")
                    .replace(".", "\\.")
                    .replace("$$$", ".*")
                    .replace("?", ".")
            })
            .filter_map(|p| regex::Regex::new(&p).ok())
            .collect::<Vec<Regex>>();

        regexes.push(Regex::new(".git").unwrap());

        RegexHelper { regexes }
    }

    pub fn check<S: AsRef<str>>(&self, str: S) -> bool {
        for r in &self.regexes {
            if r.is_match(str.as_ref()) {
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
        let ignore = RegexHelper::from_gitignore(".");

        assert!(ignore.check("haha/target"));
        assert!(ignore.check("hihi/target"));
        assert!(ignore.check("/local_data"));
        assert!(ignore.check("123/local_data"));
        assert!(ignore.check("123/local_data/1234"));
        assert!(!ignore.check("123/1234"));
    }

    #[test]
    fn check_from_string() {
        let ignore = match RegexHelper::from_string(".*some") {
            Ok(i) => i,
            Err(err) => {
                assert_eq!(err, "");
                return;
            }
        };

        assert!(ignore.check("haha/some"));
        assert!(ignore.check("asdgoasogaosomesome"));
        assert!(!ignore.check("soahasme"));
    }
}
