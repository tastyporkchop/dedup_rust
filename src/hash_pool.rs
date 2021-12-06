use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::Error;


pub struct HashPool<T> {
    pool: ThreadPool,
    rx: Sender<T>,
    tx: Receiver<T>,
}

impl <T> HashPool<T> {
    pub fn new() -> HashPool<T> {
        let (rx, tx) = channel();
        HashPool {
            pool: ThreadPool::new(4),
            rx: rx,
            tx: tx,
        }
    }

    pub fn hash(&self, filepath: &str) -> Result<String, Error> {
        let tx = self.tx.clone();
        pool.execute(move|| {
            match get_hash(filepath) {
                Ok(h) => tx.send(h).expect(""),
                Err(e) => 
            }
        });
    }

    fn get_hash(path: &str) -> Result<String, std::io::Error> {
        // open the file
        let mut file = File::open(&path)?;

        // hasher and buffer
        let mut hasher = Hasher::new(Algorithm::MD5);
        let mut buf: Vec<u8> = vec![0; 4096];

        loop {
            let n = file.read(&mut buf[..])?;
            if n == 0 {
                // eof reached
                break;
            }
            hasher.write(&buf[0..n])?;
        }

        let mut result = String::new();
        for v in hasher.finish() {
            result.push_str(&format!("{:x}", v));
        }
        Ok(result)
    }
}

