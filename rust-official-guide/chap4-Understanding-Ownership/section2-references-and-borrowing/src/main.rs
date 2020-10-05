fn main() {
    // 【ポイント1】
    // NOT Mutualな参照 -> 変更しようとするとエラーになる
    // let s = String::from("abc");
    // change(&s);
    //
    // Mutualな参照 -> 変更OK (1人用)
    let mut s = String::from("hello");
    m_change(&mut s); // 関数スコープ内で値が変わる

    // 【ポイント2】
    // ミュータブルな参照を同時に参照するとコンパイルエラーになる
	let mut u = String::from("hello");
    let r1 = &mut u;
    let r2 = &mut u; //    ====>      ^^^^^^ second mutable borrow occurs here
    println!("{}, {}", r1, r2);
    // しかし、println!("{}", r1) だけではエラーにならない

    // 【ポイント3】
    let reference_to_nothing = dangle();
}

//fn change(some_string: &String) {
    // some_string.push_str(", world");
    // 借り物なので変更できません エラー
    /*
   |     some_string.push_str(", world");
   |     ^^^^^^^^^^^ `some_string` is a `&` reference, so the data it refers to cannot be borrowed as mutable
    */
// }

fn m_change(some_string: &mut String) {
    some_string.push_str(", world");
}

// 参照を返そうとするが、関数スコープ内でその元ポインタはライフタイム的に消えるので返すことはできない。
// ==> コンパイルエラーを生成する
//fn dangle() -> &String {
//    |        ^ help: consider giving it a 'static lifetime: `&'static`
    //let s = String::from("hello");

    //&s
//}

// &String(参照)ではなく、Stringにすれば、参照ではなく、所有権を呼び出し元へムーブできるので問題なし
fn ok_dangle() -> String {
    let s = String::from("hello");
    s
}
