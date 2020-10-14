fn main() {
    // let v: Vec<i32> = Vec::new();
    let mut v = vec![1, 2, 3];

    v.push(5);
    v.push(6);

    // let third: &i32 = &v[100]; // パニックが発生する
    let third: Option<&i32> = v.get(100); // パニックではなくNoneを出す

    for i in &v { // 参照でループ
        println!("{}", i);
    }

    // 全要素を変更したいとき
    let mut v = vec![100, 32, 57];
    for i in &mut v { // 可変参照でループ
        *i += 50;
    }
}
