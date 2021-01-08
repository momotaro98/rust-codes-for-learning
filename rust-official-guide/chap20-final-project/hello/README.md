
# src/lib.rc worker new で以下にするとうまくいかない理由

```rust
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            while let Ok(job) = receiver.lock().unwrap().recv() {
                println!("Worker {} got a job; executing.", id);

                job();
            }
        });

        Worker { id, thread }
    }
}
```

> This code compiles and runs but doesn’t result in the desired threading behavior: a slow request will still cause other requests to wait to be processed. The reason is somewhat subtle: the Mutex struct has no public unlock method because the ownership of the lock is based on the lifetime of the MutexGuard<T> within the LockResult<MutexGuard<T>> that the lock method returns. At compile time, the borrow checker can then enforce the rule that a resource guarded by a Mutex cannot be accessed unless we hold the lock. But this implementation can also result in the lock being held longer than intended if we don’t think carefully about the lifetime of the MutexGuard<T>. Because the values in the while let expression remain in scope for the duration of the block, the lock remains held for the duration of the call to job(), meaning other workers cannot receive jobs.

> By using loop instead and acquiring the lock without assigning to a variable, the temporary MutexGuard returned from the lock method is dropped as soon as the let job statement ends. This ensures that the lock is held during the call to recv, but it is released before the call to job(), allowing multiple requests to be serviced concurrently.
