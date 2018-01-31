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

use std::io::{self, Error, Write};
use std::mem::transmute;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::windows::io::AsRawSocket;

use glib_sys;
use libc;

pub struct SocketReceiver(TcpStream);

impl SocketReceiver {
    pub fn to_channel(&self) -> *mut glib_sys::GIOChannel {
        let fd = self.0.as_raw_socket();
        unsafe {
            let channel = g_io_channel_win32_new_socket(fd as libc::c_int);
            glib_sys::g_io_channel_set_encoding(channel, ptr::null(), ptr::null_mut());
            channel
        }
    }
}

pub struct SocketSender(TcpStream);

impl SocketSender {
    pub fn close(&self) -> Result<(), Error> {
        self.0.shutdown(Shutdown::Both)
    }

    pub fn send<T>(&self, data: T) -> io::Result<usize> {
        let data = Box::into_raw(Box::new(data)) as i64;
        let array: [u8; 8] = unsafe { transmute(data) };
        self.0.write(&array)
    }
}

pub fn pair() -> io::Result<(SocketSender, SocketReceiver)> {
    let listener = TcpListener::bind("localhost:0")?;
    let addr = listener.local_addr()?;
    let receiver_socket = TcpStream::connect(addr)?;
    receiver_socket.set_nonblocking(true)?;
    let (sender_socket, _) = listener.accept()?;
    receiver_socket.set_nonblocking(false)?;
    Ok((SocketSender(sender_socket), SocketReceiver(receiver_socket)))
}

extern "C" {
    fn g_io_channel_win32_new_socket(socket: libc::c_int) -> *mut glib_sys::GIOChannel;
}
