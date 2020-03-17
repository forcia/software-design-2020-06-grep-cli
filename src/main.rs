use clap::{crate_authors, crate_version, App, Arg};
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
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

struct GrepResult {
    file_path: String,
    hit_lines: Vec<String>,
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
            Arg::with_name("FILES")
                .help("take PATTERNS from FILES")
                .required(true)
                .multiple(true)
                .index(2),
        )
        .get_matches();

    let pattern = matches.value_of("PATTERNS").unwrap().to_string();
    let file_pathes = matches
        .values_of("FILES")
        .unwrap()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let is_fixed_strings_mode = matches.is_present("fixed-strings");

    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];
    for file_path in file_pathes {
        let pattern = pattern.clone(); // 'static borrowの回避のため
        let tx = mpsc::Sender::clone(&tx); // 転送側を複製
        let handle = thread::spawn(move || {
            // Create a path to the desired file
            let path = Path::new(&file_path);
            let display = path.display();

            // Open the path in read-only mode, returns `io::Result<File>`
            let mut file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", display, why.to_string()),
                Ok(file) => file,
            };

            let matcher = if is_fixed_strings_mode {
                Matcher::FixedStrings(FixedStringsMatcher::new(&pattern.as_str()))
            } else {
                Matcher::ExtendedRegexp(ExtendedRegexpMatcher::new(&pattern))
            };
            let mut result = GrepResult {
                file_path: file_path.clone(),
                hit_lines: vec![],
            };
            // Read the file contents into a string, returns `io::Result<usize>`
            let mut s = String::new();
            // TODO: ディレクトリとファイルを区別する
            match file.read_to_string(&mut s) {
                Err(why) => panic!("couldn't read {}: {}", display, why.to_string()),
                Ok(_) => {
                    for line in s.lines() {
                        if match &matcher {
                            Matcher::ExtendedRegexp(m) => m.execute(line),
                            Matcher::FixedStrings(m) => m.execute(line),
                        } {
                            result.hit_lines.push(line.to_string());
                            // println!("{}", line);
                        }
                    }
                }
            }
            tx.send(result).unwrap();
        });
        handles.push(handle);
    }
    for handle in handles {
        // TODO: エラー処理をする
        let _ = handle.join().unwrap();
        let result = rx.recv().unwrap();
        // TODO: バッファリングをすると高速化できるらしい https://keens.github.io/blog/2017/10/05/rustdekousokunahyoujunshutsuryoku/
        if result.hit_lines.len() > 0 {
            println!("{}", result.file_path);
            for line in result.hit_lines {
                println!("{}", line);
            }
            println!("");
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
