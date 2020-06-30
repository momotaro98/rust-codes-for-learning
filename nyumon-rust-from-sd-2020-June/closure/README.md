
`std:thread::spawn(move || {` であるべきところを`move`を取り除いた場合、以下のようなエラーが出る。

```shell
   Compiling closure v0.1.0 (/Users/shintaro/learning/rust-sandbox/rust-codes-for-learning/nyumon-rust-from-sd-2020-June/closure)
error[E0597]: `add` does not live long enough
  --> src/main.rs:34:49
   |
33 |           .map(|vv| std::thread::spawn(|| {
   |  ______________----_-
   | |              |
   | |              value captured here
34 | |             vv.for_each(|nn| print!("{},", nn + add));
   | |                                                 ^^^ borrowed value does not live long enough
35 | |         })
   | |__________- argument requires that `add` is borrowed for `'static`
...
40 |   }
   |   - `add` dropped here while still borrowed

error: aborting due to previous error

For more information about this error, try `rustc --explain E0597`.
error: Could not compile `closure`.

To learn more, run the command again with --verbose.
```

moveは外にある値の所有権を移すのを明示している。

> moveが無い場合は所有権を借用する。この場合、その所有権を持つ変数がスレッドでのクロージャの実行が
> 終わるまで生存していることが保証されないため、このようなエラーが出ています。
