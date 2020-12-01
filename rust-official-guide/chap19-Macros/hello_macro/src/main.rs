use hello_macro::HelloMacro;

struct Pancakes;

// Not マクロな場合
impl HelloMacro for Pancakes {
    // こういった通常の場合はこのように型ごとに実装を書かないといけないし
    fn hello_macro() {
        println!("Hello, Macro! My name is Pancakes!");
        // RustにはReflectionがないので対象の型名を出力するにはマクロが必要になる
    }
}

fn main() {
    Pancakes::hello_macro();
}
