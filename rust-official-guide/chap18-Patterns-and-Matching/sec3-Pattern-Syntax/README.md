# パターンマッチの文法

## Multiple Patterns `|`

> In match expressions, you can match multiple patterns using the `|` syntax, which means or.

```rust
let x = 1;

match x {
    1 | 2 => println!("one or two"),
    3 => println!("three"),
    _ => println!("anything"),
}
```

> This code prints one or two.


## Matching Ranges of Values with `..=`

> The `..=` syntax allows us to match to an inclusive range of values.

```rust
let x = 5;

match x {
    1..=5 => println!("one through five"),
    _ => println!("something else"),
}
```

```
one through five
```

## Destructuring to Break Apart Values 構造体のフィールドを見てマッチ

> We can also use patterns to destructure structs, enums, tuples, and references

```rust
fn main() {
    let p = Point { x: 0, y: 7 };

    match p {
        Point { x, y: 0 } => println!("On the x axis at {}", x),
        Point { x: 0, y } => println!("On the y axis at {}", y),
        Point { x, y } => println!("On neither axis: ({}, {})", x, y),
    }
}
```

```
On the y axis at 7
```

> We can specify these complex conditions in one match expression, even though two enums are involved.


```rust
enum Color {
    Rgb(i32, i32, i32),
    Hsv(i32, i32, i32),
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(Color),
}

fn main() {
    let msg = Message::ChangeColor(Color::Hsv(0, 160, 255));

    match msg {
        Message::ChangeColor(Color::Rgb(r, g, b)) => println!(
            "Change the color to red {}, green {}, and blue {}",
            r, g, b
        ),
        Message::ChangeColor(Color::Hsv(h, s, v)) => println!(
            "Change the color to hue {}, saturation {}, and value {}",
            h, s, v
        ),
        _ => (),
    }
}
```


## Ignoring Values in a Pattern

### Ignoring Remaining Parts of a Value with ..

> With values that have many parts, we can use the .. syntax to use only a few parts and ignore the rest, avoiding the need to list underscores for each ignored value.

```rust
struct Point {
    x: i32,
    y: i32,
    z: i32,
}

let origin = Point { x: 0, y: 0, z: 0 };

match origin {
    Point { x, .. } => println!("x is {}", x),
}
```

## Extra Conditionals with Match Guards

> A _match guard_ is an additional `if` condition specified after the pattern in a match arm that must also match, along with the pattern matching, for that arm to be chosen.

```rust
let num = Some(4);

match num {
    Some(x) if x < 5 => println!("less than five: {}", x),
    Some(x) => println!("{}", x),
    None => (),
}
```

```
less than five: 4
```

## `@` Bindings

```rust
enum Message {
    Hello { id: i32 },
}

let msg = Message::Hello { id: 5 };

match msg {
    Message::Hello {
        id: id_variable @ 3..=7,
    } => println!("Found an id in range: {}", id_variable),
    Message::Hello { id: 10..=12 } => {
        println!("Found an id in another range")
    }
    Message::Hello { id } => println!("Found some other id: {}", id),
}
```

> This example will print Found an id in range: 5. By specifying `id_variable @` before the range `3..=7`, we’re capturing whatever value matched the range while also testing that the value matched the range pattern.
