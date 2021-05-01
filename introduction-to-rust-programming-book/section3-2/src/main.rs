// trait と dyn

trait Tweet {
    fn tweet(&self);

    // trait にはデフォルト実装を持たせることができる。
    fn tweet_twice(&self) {
        self.tweet();
        self.tweet();
    }

    fn shout(&self) {
        println!("Uoooooooooooooooohhh!!!!!!!!");
    }
}

struct Dove;
struct Duck;

impl Tweet for Dove {
    fn tweet(&self) {
        println!("Coo!");
    }
}

impl Tweet for Duck {
    fn tweet(&self) {
        println!("Quack!");
    }
}

fn main() {
    let dove = Dove {};
    dove.tweet();
    dove.tweet_twice();
    dove.shout();

    let duck = Duck {};

    // OK
    let bird_vec: Vec<Box<dyn Tweet>> = vec![Box::new(dove), Box::new(duck)];

    // NG - Compile Error
    // let bird_vec: Vec<dyn Tweet> = vec![dove, duck];
        // doesn't have a size known at compile-time
        // Vec<dyn Tweet>ではコンパイル時にサイズがわからないのでコンパイルエラー
        // BoxをつけることでポインタになるのでVec<T>に渡すことができる。

    // Warning
    // let bird_vec: Vec<Box<Tweet>> = vec![Box::new(dove), Box::new(duck)];
        // Vecの<T>にてdynを明示的にする必要がある
        // help: use `dyn`: `dyn Tweet`

    for bird in bird_vec {
        // Boxのため動的(dyn (dynamic))ディスパッチをしている。
        // Rustでは静的ディスパッチが基本。コンパイル時にインスタンスを確定させる。
        bird.tweet();
    }
}
