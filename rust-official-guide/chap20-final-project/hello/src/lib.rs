pub struct ThreadPool;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        // if size <= 0 then got > thread 'main' panicked at 'assertion failed: size > 0', src/lib.rs:5:9

        ThreadPool
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
    }
}
