use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

// Optionがなぜ必要になるかちゃんと理解できていない(2021-01-10時点)
// thread はOptionをつけないと以下のエラーになる。その原因が以下の英文。
// >    |             ^^^^^^^^^^^^^ move occurs because `worker.thread` has type `std::thread::JoinHandle<()>`, which does not implement the `Copy` trait
// > we need to move the thread out of the Worker instance that owns thread so join can consume the thread. We did this in Listing 17-15: if Worker holds an Option<thread::JoinHandle<()>> instead, we can call the take method on the Option to move the value out of the Some variant and leave a None variant in its place. In other words, a Worker that is running will have a Some variant in thread, and when we want to clean up a Worker, we’ll replace Some with None so the Worker doesn’t have a thread to run.

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);

                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);

                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        // if size <= 0 then got > thread 'main' panicked at 'assertion failed: size > 0', src/lib.rs:5:9

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        // [Note] self.sender.send と thread.join() のfor loop
        // を1つにしようとすると self.sender.send が非同期であるために、デッドロックになってしまう。
        // [原文] > To better understand why we need two separate loops, imagine a scenario with two workers. If we used a single loop to iterate through each worker, on the first iteration a terminate message would be sent down the channel and join called on the first worker’s thread. If that first worker was busy processing a request at that moment, the second worker would pick up the terminate message from the channel and shut down. We would be left waiting on the first worker to shut down, but it never would because the second thread picked up the terminate message. Deadlock!

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
