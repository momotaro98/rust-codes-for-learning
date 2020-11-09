pub trait Draw {
    fn draw(&self);
}

// trait object (`Box<dyn Draw>`) を持ったScreen構造体
pub struct Screen {
    pub components: Vec<Box<dyn Draw>>, // インタフェースを満たすTrait Objectを持つベクター
    // Vec<T>で where T: Draw
    // ではインタフェースを満たす一種類の型しかVecの中の値に入れることができない。
}

impl Screen {
    pub fn run(&self) {
        for component in self.components.iter() {
            component.draw();
        }
    }
}
