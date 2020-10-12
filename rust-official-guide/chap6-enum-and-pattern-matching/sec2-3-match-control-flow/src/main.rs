enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(String),
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter(state) => { // stateという変数へバインディング
            println!("State quarter from {}!", state);
            25
        }
    }

    // if let はmatchの糖衣構文！(Syntax Sugar) matchの方が無難に見える
    /*
    let mut count = 0;
    if let Coin::Quarter(state) = coin {
        println!("State quarter from {}", state);
    } else {
        count += 1
    }
    */
}

fn main() {
    let coin = Coin::Nickel;
    value_in_cents(coin);
    let coin = Coin::Quarter("Alaska".to_string());
    value_in_cents(coin);
}
