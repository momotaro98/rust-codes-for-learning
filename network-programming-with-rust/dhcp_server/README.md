
## DHCPサーバの動作確認について

書籍では実際に動かしていたが、ラズパイ等のLAN内でDHCPのポートを受け付けれる環境がなかったので実際には試して動作させていない。

## なぜDHCPを実装するのか


> 本章でDHCPサーバをテーマに選んだ理由は、ネットワークの学習用途に適しているからです。その根拠を次に挙げます。
> * プロトコルが複雑過ぎないため、完全準拠にこだわらなければ作りやすいから。
> * 有名なプロトコルであり、かつ一般的なネットワークにおいて必須の機能だから。
> * ネットワーク内のホストに対してIPアドレスが割り当てられるという明快な動作確認が可能だから。

`Teruya Ono. Introduction to network programming with Rust (Japanese Edition) (p. 125). Kindle Edition. `

## 構成

```
.
├── Cargo.lock
├── Cargo.toml
├── sql
│   └── create_table.sql
└── src
    ├── database.rs
    ├── dhcp.rs
    ├── main.rs
    └── util.rs
```

* `main.rs`: DHCPリクエストの待ち受け、受信、および適切なレスポンス返却の処理をする。
* `database.rs`: データベース操作の処理をまとめたモジュール
* `dhcp.rs`:  DHCPパケットやDHCPサーバで管理する情報についてまとめたモジュール
* `util.rs`: 汎用的な機能をまとめたモジュール

## DHCP 仕様 on RFC

* https://datatracker.ietf.org/doc/html/rfc2131
  * DHCPの仕様
* https://datatracker.ietf.org/doc/html/rfc2132
  * DHCPパケットのOption領域の各コードの仕様

## [note] DHCP 仕組み 概要

サーバは [./sql/create_table.sql](./sql/create_table.sql) のMACアドレスとIPアドレスのペアのテーブルを持つ。

* クライアントがBroadcastでIP割当の要求をする
* サーバは割り当て用のIPアドレスの提案レスポンスをBroadcastで返す
  * case1: 既にサーバが持つDB上に対象クライアントのMACアドレスとIPアドレスのペアがあればそれを返す
  * case2: クライアントが欲しいIPアドレスを指定していればIPプールから探してあればそれを返す
  * case3: case1とcase2を満たしていないならば、IPプールにあるIPを1つ選んで返す
  * どのケースでもICMP Echo(Ping)を使って割り当てようとしているIPアドレスが既に実際に使われていないか確認する
* クライアントはサーバからの提案を受け入れるメッセージを投げる
* クライアントからのOKのメッセージが来たらサーバは対象クライアントのMACアドレスとIPアドレスのペアをDBに保存する