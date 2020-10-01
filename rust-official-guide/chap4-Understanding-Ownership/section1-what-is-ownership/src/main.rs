fn main() {
    let s = String::from("hello");  // sがスコープに入る

    takes_ownership(s);             // ヒープにあるsが持っていた値はtakes_ownershipへムーブした
    // println!("{}", s);  // ← sを参照しようとするとコンパイルエラーになる => ^ value borrowed here after move

    let x = 5;                      // xがスコープに入る

    makes_copy(x);                  // xも関数にムーブされるが、
    println!("{}", x);  // ← i32型はCopyなので参照できる。

} // ここでxがスコープを抜け、sもスコープを抜ける。ただし、sの値はムーブされているので、何も特別なことは起こらない。

fn takes_ownership(some_string: String) { // some_stringがスコープに入る。
    println!("{}", some_string);
} // ここでsome_stringがスコープを抜け、`drop`が呼ばれる。後ろ盾してたメモリが解放される。

fn makes_copy(some_integer: i32) { // some_integerがスコープに入る
    println!("{}", some_integer);
} // ここでsome_integerがスコープを抜ける。何も特別なことはない。
