struct CustomSmartPointer {
    data: String,
}

// Drop は値がスコープを抜けるときに呼ばれるメソッド
impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer with data `{}`!", self.data);
    }
}

fn main() {
    let c = CustomSmartPointer {
        data: String::from("some data"),
    };
    println!("CustomSmartPointer created.");
    drop(c); // c.drop()だとコンパイルエラーになる。 error: explicit destructor calls not allowed
    // drop()関数を使うことでマニュアルでdrop(&mut self)メソッドを呼ぶことができる。
    println!("CustomSmartPointer dropped before the end of main.");
}
