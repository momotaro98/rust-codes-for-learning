struct RightTriangle { // 直角三角形
    base: f64,
    perpendicular: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

// ①トレイトの宣言
trait GeoCalculator {
    fn area(&self) -> f64;
    fn length(&self) -> f64;
}

// ②トレイトGeoCalculatorが要求しているメソッドのRightTriangleに対する実装
impl GeoCalculator for RightTriangle {
    fn area(&self) -> f64 {
        (self.base * self.perpendicular) * 0.5
    }
    fn length(&self) -> f64 {
        self.base + self.perpendicular
            + (self.base.powi(2) + self.perpendicular.powi(2)).sqrt()
    }
}

// ③トレイトGeoCalculatorが要求しているメソッドのRectangleに対する実装
impl GeoCalculator for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
    fn length(&self) -> f64 {
        (self.width + self.height) * 2.0
    }
}

// ④Tにトレイト境界を付与して関数定義
// fn printval<T: GeoCalculator>(poly: &T) { // これだとコンパイルエラーになる
fn printval<T: GeoCalculator>(poly: &T) { // ジェネリクスでは静的ディスパッチとなりコンパイル時に具体型のメソッドになる
// fn printval(poly: &dyn GeoCalculator) { // トレイトオブジェクトとしてもジェネリクスと同様にできる。これは動的ディスパッチとなり実行時に関数ポインタを辿るオーバーヘッドが生じる。しかし柔軟にすることができる。
    println!("{}", poly.area());
    println!("{}", poly.length());
}

fn main() {
    let tri = RightTriangle {
        base: 3.0,
        perpendicular: 4.0,
    };
    printval(&tri);
    let rec = Rectangle {
        width: 3.0,
        height: 4.0,
    };
    printval(&rec);
}
