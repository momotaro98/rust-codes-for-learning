// Enum定義
enum TempComp {
    Equal, // 同じ温度
    Higher, // より高い温度
    Lower, // より低い温度
}

// "タプル構造体" 既存型を使ったカスタム型
struct Celsius(f64);

// Copy, Clone, Debugトレイトが実装を要求するメソッドを自動導出
#[derive(Copy, Clone, Debug)]
struct Kelvin(f64);

// 定数の宣言
const T0: f64 = 273.15;
const EPS: f64 = 1.0e-10;

trait KelvinConverter {
    fn convert_to_kelvin(&self) -> Kelvin;
}

impl KelvinConverter for Kelvin {
    fn convert_to_kelvin(&self) -> Kelvin {
        self.clone() // そのままでは所有権がMoveしてしまうので、.clone()をして返す
    }
}

impl KelvinConverter for Celsius {
    fn convert_to_kelvin(&self) -> Kelvin {
        Kelvin(self.0 + T0)
    }
}

// T, SにはKelvinConverterで宣言されたメソッドが実装されているという
// トレイト境界を付与。Kelvin, Celsiusのどちらもx,yになれる。
fn comp<T: KelvinConverter, S: KelvinConverter>(x: &T, y: &S) -> TempComp {
    let x_kelvin = x.convert_to_kelvin();
    let y_kelvin = y.convert_to_kelvin();

    if (x_kelvin.0 - y_kelvin.0).abs() < EPS {
        // EPSという許容誤差内で比較
        TempComp::Equal
    } else if x_kelvin.0 > y_kelvin.0 {
        TempComp::Higher
    } else {
        TempComp::Lower
    }
}


fn main() {
    let x = Kelvin(300.0);
    let y = Celsius(30.0);

    // comp(&x, &y)が返すTempComp型によりパターンマッチ
    match comp(&x, &y) {
        TempComp::Equal => println!("Equal"),
        TempComp::Higher => println!("x is higher"),
        TempComp::Lower => println!("x is lower"),
    }
}
