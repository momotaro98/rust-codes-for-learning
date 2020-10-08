// #[derive(Debug)] が無いとき println!("{:?}", rect1) でエラー→ error[E0277]: `Rectangle` doesn't implement `std::fmt::Debug`
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

// メソッド定義の書き方
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };

    println!("{:?}", rect1);

    println!(
        "The area of the rectangle is {} square pixels.",
        rect1.area()
    );
}
