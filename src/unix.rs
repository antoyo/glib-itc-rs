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

use std::io::{self, Error};
use std::net::Shutdown;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixDatagram;

use glib_sys;

pub struct SocketReceiver(UnixDatagram);

impl SocketReceiver {
    pub fn to_channel(&self) -> *mut glib_sys::GIOChannel {
        let fd = self.0.as_raw_fd();
        unsafe { glib_sys::g_io_channel_unix_new(fd) }
    }
}

pub struct SocketSender(UnixDatagram);

impl SocketSender {
    pub fn close(&self) -> Result<(), Error> {
        self.0.shutdown(Shutdown::Both)
    }

    pub fn send(&self) -> io::Result<usize> {
        self.0.send(b"")
    }
}

pub fn pair() -> io::Result<(SocketSender, SocketReceiver)> {
    let (sender_socket, receiver_socket) = UnixDatagram::pair()?;
    Ok((SocketSender(sender_socket), SocketReceiver(receiver_socket)))
}
