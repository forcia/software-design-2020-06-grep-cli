use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn check(line:&str, pattern:&str)->bool{
    line.contains(pattern)
}

fn main() {
    // Create a path to the desired file
    let path = Path::new("./book/CONTRIBUTING.md");
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
            // "you"が含まれている行のみを表示する
            for line in s.lines(){
                if check(line, "you") {
                    println!("{}", line);
                }
            }
        },
    }


    // `file` goes out of scope, and the "hello.txt" file gets closed
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_check(){
        assert_eq!(true, check("abcdefg", "c"));
        assert_eq!(true, check("abcdefg", "fg"));
        assert_eq!(false, check("abcdefg", "Z"));
    }
}