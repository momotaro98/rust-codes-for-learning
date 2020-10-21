use std::thread;
use std::time::Duration;

// 筋トレプランサポートアプリ用の高コストな計算結果のキャッシュをする
struct Cacher<T>
where
    T: Fn(u32) -> u32,
{
    calculation: T,
    value: Option<u32>,
}

impl<T> Cacher<T>
where
    T: Fn(u32) -> u32,
{
    fn new(calculation: T) -> Cacher<T> {
        Cacher {
            calculation,
            value: None,
        }
    }

    fn value(&mut self, arg: u32) -> u32 {
        match self.value {
            Some(v) => v, // すでにある場合はキャッシュ利用
            None => {
                let v = (self.calculation)(arg);
                self.value = Some(v);
                v
            }
        }
    }
}

fn generate_workout(intensity: u32, random_number: u32) {
    let mut expensive_result = Cacher::new(|num| { // キャッシュな構造体にクロージャを渡す
        println!("calculating slowly...");
        thread::sleep(Duration::from_secs(2));
        num
    });

    if intensity < 25 {
        println!("Today, do {} pushups!", expensive_result.value(intensity)); // 呼ぶ必要があるときに高コストを処理を呼ぶ
        println!("Next, do {} situps!", expensive_result.value(intensity));
    } else {
        if random_number == 3 {
            println!("Take a break today! Remember to stay hydrated!");
        } else {
            println!(
                "Today, run for {} minutes!",
                expensive_result.value(intensity)
            );
        }
    }
}

fn main() {
    // 筋トレプランサポートアプリ
	let simulated_user_specified_value = 10;
    let simulated_random_number = 7;
    generate_workout(simulated_user_specified_value, simulated_random_number);

    // Closureの型推論
    let example_closure = |x| x;
    let s = example_closure(String::from("hello"));
    // let n = example_closure(5); // [error] expected struct `std::string::String`, found integer
    // クロージャの場合は型指定無しで型推論をする。つじつまが合わない場合はコンパイルエラーにする。

    // クロージャーのキャプチャ
    let x = 4;
    let equal_to_x = |z| z == x; // これが `fn equal_to_x(z: int32) -> bool な関数だとxを参照することはできない。
    let y = 4;
    assert!(equal_to_x(y));
}
