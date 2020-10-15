fn main() {
    let s1 = String::from("Hello, ");
    let s2 = String::from("world!");
    let s3 = s1 + &s2; // s1はムーブされ、もう使用できないことに注意

    // +演算子は下記のStringのadd関数が使われる
    /*
     * fn add(self, s: &str) -> String {
     */
    // add呼び出しで&s2を使える理由は、コンパイラが&String引数を&strに
    // 型強制してくれるためです。 addメソッド呼び出しの際、コンパイラは、
    // 参照外し型強制というものを使用し、ここでは、 &s2を&s2[..]に変える
    // ものと考えることができます。

    let s4 = String::from("abc");
    let s5 = "def".to_string();
    let s = format!("{}-{}", s4, s5); // format!マクロ便利だから使おう

    // StringはVec<u8>のラッパである。
    let hello = "Здравствуйте";
    // let answer = &hello[0]; // StringにはIndexingできない。Rustではコンパイルエラーになる。
    let s = &hello[0..4]; // 範囲指定はコンパイルエラーにならない。が範囲指定を間違えれば実行時エラーになる。

    // chars()を使うことで自然言語の文字が取れる (Golangでのrune)
    for c in "あいう".chars() {
        println!("{}", c);
    }
    /*
    あ
    い
    う
    */

    for b in "あいう".bytes() {
        println!("{}", b);
    }
    /*
    227
    129
    130
    227
    129
    132
    227
    129
    134
    */
}
