use clap::{crate_authors, crate_version, App, Arg};
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

trait MatcherTrait {
    fn execute(&self, line: &str) -> bool;
}

struct ExtendedRegexpMatcher {
    pattern: Regex,
}
impl ExtendedRegexpMatcher {
    fn new(pattern: &str) -> ExtendedRegexpMatcher {
        ExtendedRegexpMatcher {
            pattern: Regex::new(pattern).unwrap(),
        }
    }
}
impl MatcherTrait for ExtendedRegexpMatcher {
    fn execute(&self, line: &str) -> bool {
        self.pattern.is_match(line)
    }
}

struct FixedStringsMatcher<'a> {
    pattern: &'a str,
}
impl<'a> FixedStringsMatcher<'a> {
    fn new(pattern: &str) -> FixedStringsMatcher {
        FixedStringsMatcher { pattern: pattern }
    }
}
impl<'a> MatcherTrait for FixedStringsMatcher<'a> {
    fn execute(&self, line: &str) -> bool {
        line.contains(self.pattern)
    }
}

enum Matcher<'a> {
    ExtendedRegexp(ExtendedRegexpMatcher),
    FixedStrings(FixedStringsMatcher<'a>),
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

    let pattern = match matches.value_of("PATTERNS") {
        Some(p) => p,
        None => "",
    };
    let file_path = match matches.value_of("FILE") {
        Some(f) => f,
        None => "",
    };
    let matcher = if matches.is_present("fixed-strings") {
        Matcher::FixedStrings(FixedStringsMatcher::new(&pattern))
    } else {
        Matcher::ExtendedRegexp(ExtendedRegexpMatcher::new(&pattern))
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
                if match &matcher {
                    Matcher::ExtendedRegexp(m) => m.execute(line),
                    Matcher::FixedStrings(m) => m.execute(line),
                } {
                    println!("{}", line);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extended_regexp_matcher() {
        let matcher = ExtendedRegexpMatcher::new("c");
        assert_eq!(true, matcher.execute("abcdefg"));
        let matcher = ExtendedRegexpMatcher::new("fg");
        assert_eq!(true, matcher.execute("abcdefg"));
        let matcher = ExtendedRegexpMatcher::new("Z");
        assert_eq!(false, matcher.execute("abcdefg"));
        let matcher = ExtendedRegexpMatcher::new("a.c");
        assert_eq!(true, matcher.execute("abcdefg"));
        let matcher = ExtendedRegexpMatcher::new("a+.b+");
        assert_eq!(true, matcher.execute("aaa bbb"));
        let matcher = ExtendedRegexpMatcher::new("[aA][bB][cC]");
        assert_eq!(true, matcher.execute("aBc"));
        assert_eq!(true, matcher.execute("Abc"));
    }
    #[test]
    fn test_match_fix_string() {
        let matcher = FixedStringsMatcher::new("c");
        assert_eq!(true, matcher.execute("abcdefg"));
        assert_eq!(true, matcher.execute("cccc"));
        let matcher = FixedStringsMatcher::new("fg");
        assert_eq!(true, matcher.execute("abcdefg"));
        let matcher = FixedStringsMatcher::new("Z");
        assert_eq!(false, matcher.execute("abcdefg"));
        let matcher = FixedStringsMatcher::new("a.c");
        assert_eq!(false, matcher.execute("abcdefg"));
        let matcher = FixedStringsMatcher::new("a+.b+");
        assert_eq!(false, matcher.execute("aaa bbb"));
        let matcher = FixedStringsMatcher::new("[aA][bB][cC]");
        assert_eq!(false, matcher.execute("aBc"));
    }
}
