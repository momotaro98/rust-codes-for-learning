
## Function Pointers

```rust
list_of_numbers.iter().map(|i| i.to_string()).collect();
```

以下のように関数ポインタをそのままmapへ引数と渡せる

```rust
list_of_numbers.iter().map(ToString::to_string).collect();
```

> We have another useful pattern that exploits an implementation detail of tuple structs and tuple-struct enum variants. These types use () as initializer syntax, which looks like a function call. The initializers are actually implemented as functions returning an instance that’s constructed from their arguments. We can use these initializer functions as function pointers that implement the closure traits, which means we can specify the initializer functions as arguments for methods that take closures, like so:

```
fn main() {
    enum Status {
        Value(u32), // ClosureとしてのTraitを実装しているので、mapに渡せる
        Stop,
    }

    let list_of_statuses: Vec<Status> = (0u32..20).map(Status::Value).collect();
}
```

## Returning Closures

```
fn returns_closure() -> dyn Fn(i32) -> i32 {
    |x| x + 1
}
```

The code above gets error like

```
error[E0277]: the size for values of type `(dyn std::ops::Fn(i32) -> i32 + 'static)` cannot be known at compilation time
 --> src/lib.rs:1:25
  |
1 | fn returns_closure() -> dyn Fn(i32) -> i32 {
  |                         ^^^^^^^^^^^^^^^^^^ doesn't have a size known at compile-time
  |
  = help: the trait `std::marker::Sized` is not implemented for `(dyn std::ops::Fn(i32) -> i32 + 'static)`
  = note: to learn more, visit <https://doc.rust-lang.org/book/ch19-04-advanced-types.html#dynamically-sized-types-and-the-sized-trait>
  = note: the return type of a function must have a statically known size
```

> Rust doesn’t know how much space it will need to store the closure.
> This code below will compile just fine.

```
fn returns_closure() -> Box<dyn Fn(i32) -> i32> {
    Box::new(|x| x + 1)
}
```
