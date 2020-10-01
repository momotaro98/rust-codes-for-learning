fn fibo(n: i32) -> i32 {
    let v = if n == 1 || n == 2 {
        1
    } else {
        fibo(n-1) + fibo(n-2)
    };
    v
}

fn main() {
    println!("The value of number is: {}", fibo(30));
}
