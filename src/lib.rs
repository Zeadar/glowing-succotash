use std::thread::{self, JoinHandle};

pub struct ThreadPool {
    workers: Vec<Worker>,
}

impl ThreadPool {
    ///Size is number of threads in pool.
    ///Will panic if size == 0.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size != 0);

        let mut threads: Vec<Worker> = Vec::with_capacity(size);

        for id in 0..size {
            threads.push(Worker::new(id));
        }

        ThreadPool { workers: threads }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}

struct Worker {
    id: usize,
    join_handle: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize) -> Worker {
        Worker {
            id,
            join_handle: thread::spawn(|| {}),
        }
    }
}
