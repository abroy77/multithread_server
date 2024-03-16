use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct Threadpool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let handle = thread::spawn(move || loop {

            let message = receiver.lock().unwrap().recv();
            match message {

                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job()
                },
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                
                    break;
                }

            }
            
        });
        Worker { id,handle: Some(handle) }
    }
}

impl Drop for Threadpool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}

impl Threadpool {
    /// Create a new Threadpool.
    ///
    /// n = number of threads in the pool
    ///
    /// # Panics
    ///
    /// The `new` function will panic if `n==0`
    pub fn new(n: usize) -> Threadpool {
        assert!(n > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(n);
        for i in 0..n {
            workers.push(Worker::new(i, Arc::clone(&receiver)))
        }

        Threadpool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        if let Some(sender) = self.sender.as_ref().take() {
            sender.send(job).unwrap();
        }
    }
}
