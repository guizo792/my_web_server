
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

/// A thread pool for concurrent task execution.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool with the specified number of worker threads.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of worker threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the `size` is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Execute a closure in one of the worker threads.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure or function to execute in a worker thread.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        if let Some(sender) = &self.sender {
            if let Err(err) = sender.send(job) {
                eprintln!("Failed to send a job to a worker: {:?}", err);
            }
        } else {
            eprintln!("ThreadPool has been shut down, cannot execute the job.");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender.take(); // Close the sender channel to signal workers to exit.

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                if let Err(err) = thread.join() {
                    eprintln!("Error while shutting down worker {}: {:?}", worker.id, err);
                }
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Err(_) => {
                    println!("Worker {} disconnected; shutting down.", id);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool() {
        // Create a thread pool with 2 worker threads.
        let pool = ThreadPool::new(2);

        // Execute a job in the thread pool.
        pool.execute(|| {
            println!("Job 1 executed.");
        });

        // Execute another job.
        pool.execute(|| {
            println!("Job 2 executed.");
        });
    }
}
