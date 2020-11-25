# Specifying Placeholder Types in Trait Definitions with __Associated Types__

Associated Typesを利用する

```rust
pub trait Iterator {
    type Item; // ← これがあるのがAssociated Type

    fn next(&mut self) -> Option<Self:Item>;
}
```

使い方↓

```rust
impl Iterator for Counter {
    type Item = u32; // ← 具体型を渡す

    fn next(&mut self) -> Option<Self::Item> {
        // --snip--
```

Associated Typeならば1つだけの実装にすることができる(？) ← あんまりちゃんと理解できていない

Genericsとの違い！

Genericsの場合は以下

```rust
pub trait Iterator<T> {
    fn next(&mut self) -> Option<T>;
}
```

使い方が以下

```
impl Iterator<u32> for Counter {
```

いろんなパターンで実装できてしまう 

> In other words, when a trait has a generic parameter, it can be implemented for a type multiple times, changing the concrete types of the generic type parameters each time. When we use the next method on Counter, we would have to provide type annotations to indicate which implementation of Iterator we want to use.

# Default Generic Type Parameters and Operator Overloading

```rust
trait Add<Rhs=Self> { // Rhsがジェネリクスの型パラメータ。Selfは"デフォルトの"型
    type Output;

    fn add(self, rhs: Rhs) -> Self::Output;
}
```

```rust
use std::ops::Add;

struct Millimeters(u32);
struct Meters(u32);

impl Add<Meters> for Millimeters { // 型を指定している
    type Output = Millimeters;

    fn add(self, other: Meters) -> Millimeters {
        Millimeters(self.0 + (other.0 * 1000))
    }
}
```

```rust
use std::ops::Add;

#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

impl Add for Point { // デフォルトの型としてSelfのPoint
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

fn main() {
    assert_eq!(
        Point { x: 1, y: 0 } + Point { x: 2, y: 3 },
        Point { x: 3, y: 3 }
    );
}
```
