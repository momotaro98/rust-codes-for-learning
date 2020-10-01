// five 関数はRustにおいて、完璧に問題ない関数。
// セミコロンが無い部分が戻り値の式となる。
fn five() -> i32 {
    5
}

fn main() {
    let x = five();

    println!("The value of x is: {}", x);
}
