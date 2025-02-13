mod gui;
mod logger;
mod status;
mod sweep;
mod worker;

use std::sync::mpsc::channel;
use std::thread;
use worker::worker_thread;

fn main() {
    let (cmd_sender, cmd_reciever) = channel();
    let (rsp_sender, rsp_reciever) = channel();
    let worker = thread::spawn(move || {
        worker_thread(cmd_reciever, rsp_sender).unwrap();
    });

    gui::run_gui(cmd_sender, rsp_reciever);

    worker.join().unwrap();
}
