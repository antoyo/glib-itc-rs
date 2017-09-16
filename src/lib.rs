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

mod socket_pair;

use std::io::Error;
use std::mem::transmute;
use std::ptr::null_mut;

use glib::Continue;
use glib::translate::ToGlib;

use socket_pair::{SocketReceiver, SocketSender, pair};

pub fn channel() -> (Sender, Receiver) {
    let (sender_socket, receiver_socket) = pair().unwrap();
    let sender = Sender::new(sender_socket);
    let receiver = Receiver::new(receiver_socket);
    (sender, receiver)
}

pub struct Sender {
    socket: SocketSender,
}

impl Sender {
    fn new(socket: SocketSender) -> Self {
        Sender {
            socket: socket,
        }
    }

    pub fn close(&self) -> Result<(), Error> {
        self.socket.close()?;
        Ok(())
    }

    pub fn send(&mut self) {
        self.socket.send().unwrap();
    }
}

pub struct Receiver {
    channel: *mut glib_sys::GIOChannel,
    _socket: SocketReceiver,
}

impl Receiver {
    fn new(socket: SocketReceiver) -> Self {
        let channel = socket.to_channel();
        Receiver {
            channel: channel,
            _socket: socket,
        }
    }

    pub fn connect_recv<F: FnMut() -> Continue + 'static>(&mut self, callback: F) {
        let trampoline: glib_sys::GIOFunc = unsafe { transmute(io_watch_trampoline as usize) };
        let func: Box<Box<FnMut() -> Continue + 'static>> = Box::new(Box::new(callback));
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

unsafe extern "C" fn io_watch_trampoline(source: *mut glib_sys::GIOChannel, _condition: glib_sys::GIOCondition, data: *mut libc::c_void) -> libc::c_int {
    glib_sys::g_io_channel_read_unichar(source, null_mut(), null_mut());
    let func: &Box<Fn() -> Continue + 'static> = &*(data as *const _);
    func().to_glib()
}
