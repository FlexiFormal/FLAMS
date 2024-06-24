use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub fn main() {
    let (mut s,_) = get();
    let r = Receiver(s.0.new_receiver(),Arc::new(AtomicUsize::new(0)));
    for _ in 0..3 {
        r.spawn();
    }
    s.send();
    s.send();
    s.send();
    for _ in 0..3 {
        r.spawn()
    }
    s.send();
    s.send();
    s.send();
    std::thread::sleep(std::time::Duration::from_secs(2));
}

fn get() -> (Sender,Receiver) {
    let (s,r) = async_broadcast::broadcast(32);
    (Sender(s,0),Receiver(r,Arc::new(AtomicUsize::new(0))))
}
struct Sender(async_broadcast::Sender<String>,usize);
impl Sender {
    fn send(&mut self) {
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.1 += 1;
        println!("sending {}",self.1);
        self.0.try_broadcast(format!("Message {}",self.1)).unwrap();
    }
}

struct Receiver(async_broadcast::Receiver<String>,Arc<AtomicUsize>);
impl Receiver {
    fn get(&mut self) -> Result<String,async_broadcast::TryRecvError> {
        self.0.try_recv()
    }
    fn spawn(&self) {
        let mut rec = Receiver(self.0.new_receiver(),self.1.clone());
        //while rec.get().is_ok() {}
        rec.1.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
        let s = rec.1.load(std::sync::atomic::Ordering::Relaxed);
        std::thread::spawn(move || {
            println!("creating {s}");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                match rec.get() {
                    Ok(msg) => println!("{s} received {msg}"),
                    Err(_) => ()
                }
            }
        });
    }
}

