fn to_ordinal_string(n: usize) -> String {
    let s = match n % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{}{}", n, s)
}

fn myprint<T: std::fmt::Display>(msg: T) {
    // Callerから所有権を渡された変数msgはこの関数が終了した時点で
    // ライフタイムは終了し、その値はメモリ上から開放される。
    println!("{}", msg);
}

fn main() {
    for i in 1..=10 {
        println!("Hello, {} world!", i);
        println!("Hello, {} world!", to_ordinal_string(i));
    }

    let mut x = 1; // mut をつけないと再代入は不可
    x = x + 1;
    println!("{}", x);

/*
    let s = "Hello".to_string();
    let t = s; // sが持っていたアドレスの所有権がtに移る
    println!("{}", t);
    println!("{}", s); // sは何の所有権も持たないのでエラーになる。

error[E0382]: borrow of moved value: `s`
  --> src/main.rs:14:20
   |
11 |     let s = "Hello".to_string();
   |         - move occurs because `s` has type `std::string::String`, which does not implement the `Copy` trait
12 |     let t = s; // sが持っていたアドレスの所有権がtに移る
   |             - value moved here
13 |     println!("{}", t);
14 |     println!("{}", s); // sは何の所有権も持たないのでエラーになる。
   |                    ^ value borrowed here after move
*/

    let a = 1;
    let b = a; // すべてのスカラー型はCopyトレイトを持つので所有権はbに移らない。
    println!("{}", b);
    println!("{}", a);

    /*
    let s = "Hello".to_string();
    myprint(s); // sの所有権が関数内の変数に移動
    myprint(s); // sの所有権は移動し、初期化されていない変数になるのでエラーになる
    */

    let s = "Hello".to_string();
    myprint(&s); // 共有参照によって関数に渡している
    myprint(&s); // sが所有権を失わない
}
