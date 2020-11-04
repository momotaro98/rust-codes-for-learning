pub trait Messenger {
    fn send(&self, msg: &str);
}

pub struct LimitTracker<'a, T: Messenger> {
    messenger: &'a T,
    value: usize,
    max: usize,
}

impl<'a, T> LimitTracker<'a, T>
where
    T: Messenger,
{
    pub fn new(messenger: &T, max: usize) -> LimitTracker<T> {
        LimitTracker {
            messenger,
            value: 0,
            max,
        }
    }

    pub fn set_value(&mut self, value: usize) {
        self.value = value;

        let percentage_of_max = self.value as f64 / self.max as f64;

        if percentage_of_max >= 1.0 {
            self.messenger.send("Error: You are over your quota!");
        } else if percentage_of_max >= 0.9 {
            self.messenger
                .send("Urgent warning: You've used up over 90% of your quota!");
        } else if percentage_of_max >= 0.75 {
            self.messenger
                .send("Warning: You've used up over 75% of your quota!");
        }
    }
}

// テストでは以下のモックにするメソッドが想定した回数呼ばれるかを検証したいため、カウンターを持つ
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockMessenger {
        // 問題のカウンター
        //
        // sent_messages: Vec<String>, // Error!
        //
        // モックでのsend関数で以下のエラー
        /*
         ----- help: consider changing this to be a mutable reference: `&mut self`
|             self.sent_messages.push(String::from(message));
|             ^^^^^^^^^^^^^^^^^^ `self` is a `&` reference, so the data it refers to cannot be borrowed as mutable
         */
        // helpのコメントの通りMutableにしたいが、そうすると必要とするインタフェースを満たさなくなってしまう。
        // そのため、以下のように`RefCell`を利用することでコンパイル時は見逃してくれるようになり、
        // 代わりにランタイム時に二重のMutable利用時にランタイムエラーが発生する。
        sent_messages: RefCell<Vec<String>>,
    }

    impl MockMessenger {
        fn new() -> MockMessenger {
            MockMessenger {
                // sent_messages: vec![],
                sent_messages: RefCell::new(vec![]), // NG
            }
        }
    }

    impl Messenger for MockMessenger {
        fn send(&self, message: &str) {
            // self.sent_messages.push(String::from(message)); // NG
            self.sent_messages.borrow_mut().push(String::from(message));
        }
    }

    #[test]
    fn it_sends_an_over_75_percent_warning_message() {
        let mock_messenger = MockMessenger::new();
        let mut limit_tracker = LimitTracker::new(&mock_messenger, 100);

        limit_tracker.set_value(80);

        // assert_eq!(mock_messenger.sent_messages.len(), 1); // NG
        assert_eq!(mock_messenger.sent_messages.borrow().len(), 1);
    }
}
