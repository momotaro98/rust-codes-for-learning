# ゼロコスト抽象化

# マーカトレイト

> トレイトの中には、メソッドのない、それぞれの意味を持つ意味や役割をしるしのように付与するものがあります。そのようなトレイトを、マーカトレイトといいます。CopyやSizedなどのトレイトがそれにあたります。

* Copy: 値の所有権を渡す代わりに、値のコピーを行うようにする(コピーセマンティクス)
* Send: スレッド境界を越えて所有権を転送できることを示す
* Sized: メモリ上でサイズが決まっていることを示す
* Sync: スレッド間で安全に参照を共有できることを示す

> これらのトレイトはデータを含んでいないため、実行時にもメモリ内にデータが存在しませんが、コンパイラが安全性の検査や最適化をする際に使用します。
