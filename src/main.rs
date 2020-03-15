use clap::{crate_authors, crate_version, App, Arg};
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
enum MatchMode {
    ExtendedRegexp,
    FixedStrings,
}

fn match_pattern(line: &str, pattern: &str) -> bool {
    let re = Regex::new(pattern).unwrap();
    re.is_match(line)
}
fn match_fix_string(line: &str, pattern: &str) -> bool {
    line.contains(pattern)
}

fn main() {
    let matches = App::new("grep")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Search for PATTERNS in each FILE.")
        .arg(
            Arg::with_name("fixed-strings")
                .short("F")
                .long("fixed-strings")
                .help("PATTERNS are strings"),
        )
        .arg(
            Arg::with_name("PATTERNS")
                .help("use PATTERNS for matching")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("FILE")
                .help("take PATTERNS from FILE")
                .required(true)
                .index(2),
        )
        .get_matches();

    let match_mode = if matches.is_present("fixed-strings") {
        MatchMode::FixedStrings
    } else {
        MatchMode::ExtendedRegexp
    };

    let pattern = match matches.value_of("PATTERNS") {
        Some(p) => p,
        None => "",
    };
    let file_path = match matches.value_of("FILE") {
        Some(f) => f,
        None => "",
    };

    // Create a path to the desired file
    let path = Path::new(&file_path);
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.to_string()),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why.to_string()),
        Ok(_) => {
            for line in s.lines() {
                match match_mode {
                    MatchMode::ExtendedRegexp => {
                        if match_pattern(line, &pattern) {
                            println!("{}", line);
                        }
                    }
                    MatchMode::FixedStrings => {
                        if match_fix_string(line, &pattern) {
                            println!("{}", line);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_match_pattern() {
        assert_eq!(true, match_pattern("abcdefg", "c"));
        assert_eq!(true, match_pattern("abcdefg", "fg"));
        assert_eq!(false, match_pattern("abcdefg", "Z"));
        assert_eq!(true, match_pattern("abcdefg", "a.c"));
        assert_eq!(true, match_pattern("aaa bbb", "a+.b+"));
        assert_eq!(true, match_pattern("aBc", "[aA][bB][cC]"));
    }
    #[test]
    fn test_match_fix_string() {
        assert_eq!(true, match_fix_string("abcdefg", "c"));
        assert_eq!(true, match_fix_string("abcdefg", "fg"));
        assert_eq!(false, match_fix_string("abcdefg", "Z"));
        assert_eq!(false, match_fix_string("abcdefg", "a.c"));
    }
}
