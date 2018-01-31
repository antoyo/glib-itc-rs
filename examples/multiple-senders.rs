/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

extern crate glib;
extern crate glib_itc;
extern crate gtk;

use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use glib::Continue;
use glib_itc::channel;

fn main() {
    gtk::init().unwrap();

    let num = AtomicUsize::new(0);
    let (sender, mut receiver) = channel();
    let sender = Arc::new(Mutex::new(sender));
    {
        let sender = sender.clone();
        thread::spawn(move || {
            for _ in 0..3 {
                {
                    let mut sender = sender.lock().unwrap();
                    sender.send(());
                }
                println!("500ms");
                thread::sleep(Duration::from_millis(500));
            }
        });
    }
    thread::spawn(move || {
        for _ in 0..5 {
            {
                let mut sender = sender.lock().unwrap();
                sender.send(());
            }
            println!("Send");
            thread::sleep(Duration::from_millis(1000));
        }
        println!("End");
        let mut sender = sender.lock().unwrap();
        sender.send(());
    });
    receiver.connect_recv(move |()| {
        let value = num.load(Relaxed) + 1;
        num.store(value, Relaxed);
        println!("Received");
        if value > 8 {
            gtk::main_quit();
            Continue(false)
        }
        else {
            Continue(true)
        }
    });
    gtk::main();
}
