use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // mpsc から チャネルを受け取る
    let (tx, rx) = mpsc::channel(); // let でタプルを受け取るやり方

    // move にすることでtxの所有権を取る
    thread::spawn(move || {
        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];

        for val in vals {
            tx.send(val).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    for received in rx {
        println!("Got: {}", received);
    }
}
