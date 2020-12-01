use hello_macro::HelloMacro;
use hello_macro_derive::HelloMacro;

#[derive(HelloMacro)]
struct Pancakes; // コンパイル時にマクロがHelloMacroインターフェース実装コードを生成してくれる

fn main() {
    Pancakes::hello_macro();
}
