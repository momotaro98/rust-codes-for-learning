pub trait Draw {
    fn draw(&self);
}

// trait object (`Box<dyn Draw>`) を持ったScreen構造体
pub struct Screen {
    pub components: Vec<Box<dyn Draw>>, // インタフェースを満たすTrait Objectを持つベクター
    // [VS] Vec<T>で where T: Draw ← Using generics and Trait Bounds
    // 上記はインタフェースを満たす一種類の型しかVecの中の値に入れることができない。
    // Trait Object ← Dynamic Dispatch: 実行時にどの具象メソッドかを見に行くので↓より遅い
    // Using generics and Trait Bounds ← Static Dispatch: コンパイル時にどのメソッドかがわかり速い
}

impl Screen {
    pub fn run(&self) {
        for component in self.components.iter() {
            component.draw();
        }
    }
}
