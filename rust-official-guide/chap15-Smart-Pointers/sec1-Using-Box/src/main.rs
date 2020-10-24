// Cons という型はLisp言語が元ネタになっている今では利用されていない型っぽい
// RustではVec<T>が使われているとのこと

// NG なパターン
/*
enum List {
    Cons(i32, List), // 再帰的なためコンパイラはどの程度メモリを確保すれば良いかわからずエラーになる
    Nil,
}

use crate::List::{Cons, Nil};

fn main() {
    let list = Cons(1, Cons(2, Cons(3, Nil)));
}

↓エラーメッセージ
// ^^^^^^^^^ recursive type has infinite size
2 |     Cons(i32, List),
  |               ---- recursive without indirection
  |
  = help: insert indirection (e.g., a `Box`, `Rc`, or `&`) at some point to make `List` representable
*/

// OK なパターン
enum List {
    Cons(i32, Box<List>), // Boxにすればポインタなのでポインタ分のメモリ確保となりコンパイルできる
    Nil,
}

use crate::List::{Cons, Nil};

fn main() {
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
}

