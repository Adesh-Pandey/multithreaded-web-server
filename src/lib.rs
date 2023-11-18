use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

enum Message {
    JOB_MESSAGE(Job),
    TERMINATE,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        // panic if thread cound is 0 because it does not make sense
        assert!(size > 0);

        let (sender, reciever) = mpsc::channel();

        let mut workers = Vec::with_capacity(size);
        let reciever = Arc::new(Mutex::new(reciever));
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&reciever)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::JOB_MESSAGE(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::TERMINATE).unwrap();
        }
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    pub fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = reciever.lock().unwrap().recv().unwrap();

            match message {
                Message::JOB_MESSAGE(job) => {
                    println!("running job in thread {}", id);
                    job();
                }
                Message::TERMINATE => {
                    println!("{} was said to terminate", id);
                    break;
                }
            }

            println!("Work at {} ended", id);
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
type Job = Box<dyn FnOnce() + Send + 'static>;
