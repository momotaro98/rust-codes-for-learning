use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0)); // Arc<Mutex<T>> でTがintの型
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap(); // Rc<T>では平行化で渡せないがArc<T>なら渡せる
            // [my note] Rc<T>とArc<T>が分かれるのは完全にシングルスレッドならばパフォーマンス的にRc<T>が優れるため

            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap(); // join は非同期処理すべてを待っている
    }

    println!("Result: {}", *counter.lock().unwrap());
}

