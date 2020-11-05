> Here is a recap of the reasons to choose Box<T>, Rc<T>, or RefCell<T>:

* Rc<T> enables multiple owners of the same data; Box<T> and RefCell<T> have single owners.
* Box<T> allows immutable or mutable borrows checked at compile time; Rc<T> allows only immutable borrows checked at compile time; RefCell<T> allows immutable or mutable borrows checked at runtime.
* Because RefCell<T> allows mutable borrows checked at runtime, you can mutate the value inside the RefCell<T> even when the RefCell<T> is immutable.

# まとめ

> この章は、スマートポインタを使用してRustが既定で普通の参照に対して行うのと異なる保証や代償を行う方法を講義しました。 Box<T>型は、既知のサイズで、ヒープに確保されたデータを指します。Rc<T>型は、ヒープのデータへの参照の数を追跡するので、 データは複数の所有者を保有できます。内部可変性のあるRefCell<T>型は、不変型が必要だけれども、 その型の中の値を変更する必要がある時に使用できる型を与えてくれます; また、コンパイル時ではなく実行時に借用規則を強制します。

> DerefとDropトレイトについても議論しましたね。これらは、スマートポインタの多くの機能を可能にしてくれます。 メモリリークを引き起こす循環参照とWeak<T>でそれを回避する方法も探究しました。

