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
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

use glib::Continue;
use glib_itc::channel;

#[test]
fn test() {
    gtk::init().unwrap();

    let num = Arc::new(AtomicUsize::new(0));
    let (mut sender, mut receiver) = channel();
    thread::spawn(move || {
        for _ in 0..5 {
            sender.send();
        }
        sender.send();
    });
    {
        let num = num.clone();
        receiver.connect_recv(move || {
            println!("Receive");
            let value = num.fetch_add(1, Relaxed);
            if value >= 5 {
                gtk::main_quit();
                Continue(false)
            }
            else {
                Continue(true)
            }
        });
    }
    gtk::main();

    assert_eq!(num.load(Relaxed), 6);
}
