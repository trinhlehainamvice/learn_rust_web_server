use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

// Hold pool of threads
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..size)
            .map(|i| Worker::new(i, Arc::clone(&receiver)))
            .collect();
        Self {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("ThreadPool dropping");
        println!("Waiting for workers to finish");
        self.sender.take().unwrap();
        for worker in &mut self.workers {
            worker.thread.take().unwrap().join().unwrap();
        }
    }
}

// Send job to thread in pool
struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        Self {
            id,
            thread: Some(thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv();
                if let Ok(job) = job {
                    println!("Worker {} got a job and executing.", id);
                    job();
                }
            })),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[cfg(test)]
mod tests {

    #[test]
    fn test_channel() {
        let (sender, receiver) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            sender.send(1).unwrap();
        });

        // Receiver will block until received a value from the sender
        let value = receiver.recv().unwrap();
        assert_eq!(value, 1);
    }
}
