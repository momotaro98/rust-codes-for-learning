enum List {
    Cons(i32, Rc<List>), // Cons(i32, Box<List>) だとOwnerが変わることになり下記ではb = a, c = aでコンパイルエラーになる。
    Nil,
}

use crate::List::{Cons, Nil};
use std::rc::Rc;

fn main() {
    let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
    let b = Cons(3, Rc::clone(&a)); // ポインタのコピー(Clone)をしているのでポインタの先はa, b, c すべてが同じものを指している。
    let c = Cons(4, Rc::clone(&a));

    // Reference count
    // Rc<List>がReference Countを持っている。
    let d = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
    println!("count after creating d = {}", Rc::strong_count(&d));
    let e = Cons(3, Rc::clone(&d));
    println!("count after creating e = {}", Rc::strong_count(&d));
    {
        let f = Cons(4, Rc::clone(&d));
        println!("count after creating f = {}", Rc::strong_count(&d));
    }
    println!("count after f goes out of scope = {}", Rc::strong_count(&d));
}

