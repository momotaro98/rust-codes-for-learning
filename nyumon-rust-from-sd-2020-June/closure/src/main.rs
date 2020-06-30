fn main() {
    // クロージャを利用したマルチスレッド化の例

    // スレッド数
    const N_THREADS : usize = 3;
    // 処理する整数のRange
    let series_range = 0..30;
    let add = 1;

    // ①
    // (①の説明) series_rangeをN_THREADS個に分割する。it.skip(ii)はitの先頭からii個をスキップする。
    // これにより、series_range.clone().skip(0), series_range.clone().skip(1), series_range.clone().skip(2) の3つができる
    // it.step_by(N_THREADS)はitからN_THREADS個ごとに取り出す。
    // https://doc.rust-lang.org/std/iter/trait.Iterator.html
    // の公式ドキュメントで詳細を参照できる。
    // 結果、chunks = [[0,3,6,...], [1,4,7,...], [2,5,8,...]]; が得られる。
    // ただし、mapは遅延処理なので、この段階では評価されない。
    // このmap内のクロージャは②の最後にあるcollectが呼ばれた際に、②にある他のmapとあわせて評価される。
    // series_rangeはcloneする。cloneをしないと、map内のクロージャにあるseries_rangeに所有権が移動し、
    // ii:1のときの後続で失敗してしまう。
    let chunks = (0..N_THREADS)
        .map(|ii| series_range.clone().skip(ii).step_by(N_THREADS));

    // ②
    // (②の説明) chunksの [0,3,6,...], [1,4,7,...], [2,5,8,...]
    // のそれぞれの処理を行うスレッドを起動して、それぞれの配列の要素にaddを加えた値を出力する。
    // mapは遅延処理で実際に必要になるときまでmap内のクロージャが実行されないので、
    // collectでそれぞれの要素を要求して実行させている。
    // スレッドを起動するspawnのクロージャにはmoveを付けている。これは、
    // スコープ内にある変数(この場合はadd)の所有権を強制的にクロージャ内に移動させる。
    let handles : Vec<_> = chunks
        // .map(|vv| std::thread::spawn(move || {
        .map(|vv| std::thread::spawn(|| {
            vv.for_each(|nn| print!("{},", nn + add));
        })
        ).collect();

    // ③各スレッドの終了を待つ
    handles.into_iter().for_each(|hh| hh.join().unwrap())
}
