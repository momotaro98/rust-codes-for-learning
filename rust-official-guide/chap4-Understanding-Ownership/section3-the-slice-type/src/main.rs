// &String ← 文字列参照
// &str ← 文字列スライス

fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }

    &s[..]
}

fn main() {
    let mut s = String::from("hello world");

    let word = first_word(&s);

    s.clear(); // error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
    // first_word関数↑からs参照のwordとして、参照させてかつprint↓で参照を利用しようとしている。
    // そのため参照先のsをMutable参照としてclearメソッドに渡そうとするとコンパイルエラーになる。
    println!("the first word is: {}", word);
}
