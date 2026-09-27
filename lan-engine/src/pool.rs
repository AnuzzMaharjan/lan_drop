use std::sync::{Arc, Mutex, mpsc};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<std::thread::JoinHandle<()>>
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = std::thread::spawn(move || loop {
            let job = receiver.lock().expect("Mutex poisoned").recv();

            match job {
                Ok(job) => {
                    job();
                },
                Err(_) => {
                    break;
                }
            }
        });

        Worker {
            id,
            thread:Some(thread)
        }

    }
}
impl ThreadPool {
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
            sender:Some(sender)
        }
    }
}

impl ThreadPool {
    pub fn execute<F>(&self, f:F) -> mpsc::Receiver<()>
    where
        F: FnOnce() + Send + 'static
    {
        let (done_tx, done_rx) = mpsc::channel::<()>();
        let job = Box::new(move || {
            f();
            let _ = done_tx.send(());
        });
        self.sender.as_ref().unwrap().send(job).expect("Failed to send job to worker");
        done_rx
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}