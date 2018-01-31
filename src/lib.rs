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
use std::marker::PhantomData;
use std::mem::transmute;
use std::ptr::null_mut;

use glib::Continue;
use glib::translate::ToGlib;

use socket_pair::{SocketReceiver, SocketSender, pair};

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (sender_socket, receiver_socket) = pair().unwrap();
    let sender = Sender::new(sender_socket);
    let receiver = Receiver::new(receiver_socket);
    (sender, receiver)
}

pub struct Sender<T> {
    socket: SocketSender,
    _phantom: PhantomData<T>,
}

impl<T> Sender<T> {
    fn new(socket: SocketSender) -> Self {
        Sender {
            socket: socket,
            _phantom: PhantomData,
        }
    }

    pub fn close(&self) -> Result<(), Error> {
        self.socket.close()?;
        Ok(())
    }

    pub fn send(&mut self, data: T) {
        self.socket.send(data).unwrap();
    }
}

pub struct Receiver<T> {
    channel: *mut glib_sys::GIOChannel,
    _socket: SocketReceiver,
    _phantom: PhantomData<T>,
}

impl<T> Receiver<T> {
    fn new(socket: SocketReceiver) -> Self {
        let channel = socket.to_channel();
        Receiver {
            channel: channel,
            _socket: socket,
            _phantom: PhantomData,
        }
    }

    pub fn connect_recv<F: FnMut(T) -> Continue + 'static>(&mut self, callback: F) {
        let trampoline: glib_sys::GIOFunc = unsafe { transmute(io_watch_trampoline::<T> as usize) };
        let func: Box<Box<FnMut(T) -> Continue + 'static>> = Box::new(Box::new(callback));
        let user_data: *mut libc::c_void = Box::into_raw(func) as *mut _;
        unsafe { glib_sys::g_io_add_watch(self.channel, glib_sys::G_IO_IN, trampoline, user_data) };
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        unsafe { glib_sys::g_io_channel_shutdown(self.channel, 0, null_mut()) };
        unsafe { glib_sys::g_io_channel_unref(self.channel) };
    }
}

unsafe extern "C" fn io_watch_trampoline<T>(source: *mut glib_sys::GIOChannel, _condition: glib_sys::GIOCondition,
                                            data: *mut libc::c_void) -> libc::c_int
{
    let mut buffer = [0u8; 8];
    let mut read: usize = 0;
    use std::ptr;
    let error = ptr::null_mut();
    glib_sys::g_io_channel_read_chars(source, buffer.as_mut_ptr(), 8, &mut read, error);
    assert!(error.is_null(), "No reading error in glib-itc");
    assert_eq!(read, 8, "Unable to read 8 bytes in glib-itc");
    let arg: Box<T> = Box::from_raw(transmute(buffer));
    let func: &Box<Fn(T) -> Continue + 'static> = &*(data as *const _);
    func(*arg).to_glib()
}
