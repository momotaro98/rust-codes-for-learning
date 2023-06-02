
# Overview

ポートスキャンツール → 対象のホストにて開いているポート番号を知りたい

セキュリティ攻撃の前段階にあたる。

# How to run

【注意】対象ホストは自分の管理下のホストにしないと他所様に攻撃することになる。

対象を自分のデフォルトゲートウェイにするとき、そのIPアドレスを知りたい。

```
$ route get default | grep gateway
    gateway: aterm.me
$ nslookup aterm.me
Address: 192.168.10.1
```

実行

```
$ cargo build
$ sudo ./target/debug/port-scanner 192.168.10.1 sS
# sS はTCPの SYN コントローラフラグである。他のものはコード参照。
```

【My Note】自宅のデフォルトゲートウェイを対象にしたが送信結果が返って来なかった。

コードにバグがあるかデフォルトゲートウェイ側が返さないようにしていると思われる。