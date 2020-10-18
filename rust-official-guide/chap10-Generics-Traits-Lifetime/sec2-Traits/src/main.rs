// trait は 他言語のインタフェースとほぼ同じもの
pub trait Summary {
    fn summarize_author(&self) -> String;

    // RustのTraitではデフォルト実装ができる
    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self.summarize_author())
    }
}

pub struct Tweet {
    pub username: String,
    pub content: String,
    pub reply: bool,
    pub retweet: bool,
}

impl Summary for Tweet {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }
    // summarize はデフォルト実装を利用してここでは書かない
}

fn main() {
    // Trait Basic
    let tweet = Tweet {
        username: String::from("horse_ebooks"),
        content: String::from(
            "of course, as you probably already know, people",
        ),
        reply: false,
        retweet: false,
    };
    println!("1 new tweet: {}", tweet.summarize());

    // Trait with Generics Parameter
    let number_list = vec![34, 50, 25, 100, 65];
    let result = largest(&number_list);
    println!("The largest number is {}", result);
    let char_list = vec!['y', 'm', 'a', 'q'];
    let result = largest(&char_list);
    println!("The largest char is {}", result);
}

// PartialOrd という比較演算子が使えるよーのトレイトとCopyができるよのトレイト両方を
// 持つTという型を受け付ける。これにより実装も中の比較と代入ができる。
fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut largest = list[0]; // Copyが無いとコンパイルエラーになる

    for &item in list {
        if item > largest { // PartialOrdが無いとコンパイルエラーになる
            largest = item;
        }
    }

    largest
}

