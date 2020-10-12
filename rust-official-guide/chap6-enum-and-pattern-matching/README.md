# Option<T> について

```rust
enum Option<T> {
    Some(T),
    None
}
```

Rust はNullの代わりにOption<T>を採用した。

Option<T> にはメソッドがいっぱいある
