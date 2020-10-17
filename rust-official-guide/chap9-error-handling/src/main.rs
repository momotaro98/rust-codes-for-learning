
use std::fs::File;
use std::io::ErrorKind;

fn main() {
    let f = File::open("hello.txt");

    //// 下記のList4とList5は等価な処理
    //// やりたいこと → ファイルをOpen, なければ新規にファイル作成

    // List 4 matchを使う場合
    let f = match f {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create("hello.txt") {
                Ok(fc) => fc,
                Err(e) => panic!("Program creating the file: {:?}", e),
            },
            other_error => {
                panic!("Problem opening the file: {:?}", other_error)
            }
        },
    };

    // List 5 unwrap_or_elseを使う場合
    let f = File::open("hello.txt").unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            File::create("hello.txt").unwrap_or_else(|e| {
                panic!("Program creating the file: {:?}", e);
            })
        } else {
            panic!("Problem opening the file: {:?}", error.kind());
        }
    });
}

use std::io;
use std::io::Read;

/*
enum Result<T, E> {
    Ok(T),
    Err(E),
}
*/

fn read_user_name_from_file() -> Result<String, io::Error> {
    //// 下記のList6, 7, 8, 9 は等価
    //// やりたいこと → hello.txtをOpenしその中身の文字列を返す関数を実装

    // List 6
    /*
    let f = File::open("hello.txt");
    let mut f = match f {
        Ok(file) => file,
        Err(e) => return Err(e),
    }
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
    */

    /*
    // List 7 `?`を使う. `?`は失敗時に勝手にErr(e)を返してくれる。
    // `?`はOption<T>かResult<T, E>を返す関数でしか利用できない。
    let mut f = file::open("hello.txt")?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
    */

    // List 8  List7を更にシンプルにした
    let mut s = String::new();
    File::open("hello.txt")?.read_to_string(&mut s)?;
    Ok(s)
}
