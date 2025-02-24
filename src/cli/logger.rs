use std::{
    io::{self, Write},
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

// A struct used for logging progress to the console with an animated ellipsis.
pub struct Logger {
    tx: Sender<()>,
    msg: &'static str,
}

impl Logger {
    pub fn start(msg: &'static str) -> Self {
        let (tx, rx) = mpsc::channel();
        let mut ellipses = vec!["   ", ".  ", ".. ", "..."].into_iter().cycle();

        thread::spawn(move || loop {
            match rx.try_recv() {
                Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    print!("\r[tap]: {}{} ", msg, ellipses.next().unwrap());
                    io::stdout().flush().unwrap_or_default();
                    thread::sleep(Duration::from_millis(300));
                }
            }
        });

        Logger { tx, msg }
    }

    pub fn stop(&self) {
        let _ = self.tx.send(());
        println!("\r[tap]: {}...", self.msg);
        println!("[tap]: done!");
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.tx.send(());
    }
}
