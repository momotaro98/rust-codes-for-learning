
## 作ったもの

I/O多重化とノンブロッキングI/Oを利用して複数接続を並行に処理するWebサーバ

## How to run

```
$ cargo run 127.0.0.1:8080

   Compiling webserver v0.1.0 (/Users/jp31097/workspace/github.com/momotaro98/rust-codes-for-learning/network-programming-with-rust/webserver)
    Finished dev [unoptimized + debuginfo] target(s) in 0.83s
     Running `target/debug/webserver '127.0.0.1:8080'`
[2023-06-05T15:34:36Z DEBUG webserver] Connection from 127.0.0.1:61551
[2023-06-05T15:34:36Z DEBUG webserver] readable conn_id: 1
[2023-06-05T15:34:36Z ERROR webserver] Is a directory (os error 21)
[2023-06-05T15:34:36Z DEBUG webserver] Connection from 127.0.0.1:61553
[2023-06-05T15:34:51Z DEBUG webserver] readable conn_id: 1
[2023-06-05T15:34:51Z DEBUG webserver] readable conn_id: 2
[2023-06-05T15:34:51Z DEBUG webserver] Connection from 127.0.0.1:61557
[2023-06-05T15:34:51Z DEBUG webserver] writable conn_id: 2
[2023-06-05T15:34:51Z DEBUG webserver] readable conn_id: 3
[2023-06-05T15:34:51Z DEBUG webserver] writable conn_id: 3
```