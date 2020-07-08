use clap::{crate_authors, crate_version, App, Arg};
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;


fn main() {
    println!("Hello, world!");

    let matches = App::new("grep")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Search for PATTERNS in each FILE!!!")
        .arg(
            Arg::with_name("fixed-strings")
                .short("F")
                .long("fixed-strings")
                .help("PATTERNS are strings"),
        )
        .arg(
            Arg::with_name("PATTERNS")
                .help("PATTERNS are strings")
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

    let pattern = matches.value_of("PATTERNS").unwrap();
    let file_paths = matches
        .values_of("FILES")
        .unwrap()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let is_fixed_strings_mode = matches.is_present("fixed-strings");
    println!("{:?}", matches); // 動作確認用
    println!("{:?}", pattern); // 動作確認用
    println!("{:?}", file_paths); // 動作確認用
    println!("{:?}", is_fixed_strings_mode); // 動作確認用

    for file_path in file_paths {
        let path = Path::new(&file_path);
        let display = path.display(); // 表示用の文字列を取得する
        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why.to_string()),
            Ok(file) => file,
        };
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display, why.to_string()),
            Ok(_) => {
                for line in s.lines() {
                    println!("{}", line);
                }
            }
        }
    }
}
