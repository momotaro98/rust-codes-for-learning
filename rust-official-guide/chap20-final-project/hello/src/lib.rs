use std::thread;

pub struct ThreadPool {
    threads: Vec<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        // if size <= 0 then got > thread 'main' panicked at 'assertion failed: size > 0', src/lib.rs:5:9

        let mut threads = Vec::with_capacity(size);

        for _ in 0..size {
            // create some threads and store them in the vector
        }

        ThreadPool { threads }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
    }
}
