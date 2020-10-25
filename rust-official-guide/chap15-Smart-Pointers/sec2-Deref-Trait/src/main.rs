struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

use std::ops::Deref;

// Deref は`*`のデリファレンス演算子が利用するためのメソッド
impl<T> Deref for MyBox<T> {
    type Target = T; // 特別な文法。Associated types という。詳細は19章で解説。
    // [↑をコメントアウトにしたとき] → error: not all trait items implemented, missing: `Target`

    fn deref(&self) -> &T {
        &self.0
    }
}

fn main() {
    let x = 5;
    let y = MyBox::new(x);

    assert_eq!(5, x);
    assert_eq!(5, *y);
    /*
    // When we entered *y in,  behind the scenes Rust actually ran this code:
    (y.deref())
    */
}
