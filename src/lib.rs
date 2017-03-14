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
extern crate glib_sys;
extern crate libc;
extern crate rand;
extern crate unix_socket;

use std::env::temp_dir;
use std::fs::remove_file;
use std::os::unix::io::AsRawFd;
use std::mem::transmute;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;

use glib::Continue;
use glib::translate::ToGlib;
use rand::random;
use unix_socket::UnixDatagram;

pub fn channel() -> (Sender, Receiver) {
    let sender = Sender::new();
    let receiver = Receiver::new(&sender.path);
    (sender, receiver)
}

pub struct Sender {
    path: PathBuf,
    socket: UnixDatagram,
}

impl Sender {
    fn new() -> Self {
        let mut temp_dir = temp_dir();
        temp_dir.push(format!("socket{}", random::<u32>()));
        Sender {
            path: temp_dir,
            socket: UnixDatagram::unbound().unwrap(),
        }
    }

    pub fn send(&self) {
        self.socket.send_to(b"", &self.path).unwrap();
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        remove_file(&self.path).ok();
    }
}

pub struct Receiver {
    channel: *mut glib_sys::GIOChannel,
    socket: UnixDatagram,
}

impl Receiver {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        Receiver {
            channel: null_mut(),
            socket: UnixDatagram::bind(path).unwrap(),
        }
    }

    pub fn connect_recv<F: Fn() -> Continue + 'static>(&mut self, callback: F) {
        let fd = self.socket.as_raw_fd();
        self.channel = unsafe { glib_sys::g_io_channel_unix_new(fd) };
        let trampoline: glib_sys::GIOFunc = unsafe { transmute(io_watch_trampoline as usize) };
        let func: Box<Box<Fn() -> Continue + 'static>> = Box::new(Box::new(callback));
        let user_data: *mut libc::c_void = Box::into_raw(func) as *mut _;
        unsafe { glib_sys::g_io_add_watch(self.channel, glib_sys::G_IO_IN, trampoline, user_data) };
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        unsafe { glib_sys::g_io_channel_shutdown(self.channel, 0, null_mut()) };
        unsafe { glib_sys::g_io_channel_unref(self.channel) };
    }
}

unsafe extern "C" fn io_watch_trampoline(source: *mut glib_sys::GIOChannel, condition: glib_sys::GIOCondition, data: *mut libc::c_void) -> libc::c_int {
    let status = unsafe { glib_sys::g_io_channel_read_unichar(source, null_mut(), null_mut()) };
    let func: &Box<Fn() -> Continue + 'static> = &*(data as *const _);
    func().to_glib()
}
