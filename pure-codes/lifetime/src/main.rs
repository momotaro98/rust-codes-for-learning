/*

// コンパイルエラーになる関数
fn get_longest_string(string1: &str, string2: &str) -> &str {
    if string1.len() >= string2.len() {
        string1
    } else {
        string2
    }
}

【Note】&str型の参照された引数の値(borrowed value)をそのまま返すような関数では、Lifetimeパラメータが無いという理由で以下のようなコンパイルエラーになる。

-----------
4 | fn get_longest_string(string1: &str, string2: &str) -> &str {
  |                                 ----           ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `string1` or `string2`
help: consider introducing a named lifetime parameter
  |
4 | fn get_longest_string<'a>(string1: &'a str, string2: &'a str) -> &'a str {
-----------

 */

fn get_longest_string<'a>(string1: &'a str, string2: &'a str) -> &'a str {
    if string1.len() >= string2.len() {
        string1
    } else {
        string2
    }
}

fn main() {
    let string1 = String::from("hello");
    let result;
    /*
    // コンパイルエラーになる呼び出し
    { // スコープを作成
        let string2 = String::from("world");
        result = get_longest_string(&string1, &string2);
    }

    【Note】get_longest_string に渡した引数と返り値のLifetime(生きるスコープの範囲)が
            &string2とresultで異なっているので以下のコンパイルエラーになる。

    ----------
    error[E0597]: `string2` does not live long enough
    --> src/main.rs:39:47
   |
38 |         let string2 = String::from("world");
   |             ------- binding `string2` declared here
39 |         result = get_longest_string(&string1, &string2);
   |                                               ^^^^^^^^ borrowed value does not live long enough
40 |     }
   |     - `string2` dropped here while still borrowed
...
66 |     println!("The longest string is: {}", result);
   |                                           ------ borrow later used here
    ----------
    */

    let string2 = String::from("world");

    result = get_longest_string(&string1, &string2);

    println!("The longest string is: {}", result);
}

