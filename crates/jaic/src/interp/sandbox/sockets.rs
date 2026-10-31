//! The sockets a `-os wasm` run can use: listeners the host pre-opened, and the connections they accept.
//!
//! A WASI program cannot create a socket; it can only `sock_accept` on a listener the host
//! pre-opened and then `sock_recv`, `sock_send` and `sock_shutdown`. The standard library asks for
//! them by their C names (`accept`, `recv`, `send`, `shutdown`), and the answers here follow what
//! the WASI runtime (`stdlib/Extensions/Wasi_Runtime/sockets.jai`) makes of those names.

use super::{EBADF, EINVAL, LibcFn, SandboxHost, read_bytes, write_bytes, write_u32};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};

const ENOTSOCK: u64 = 88;
const ENOTCONN: u64 = 107;
const EPIPE: u64 = 32;
const ECONNRESET: u64 = 104;
const EIO: u64 = 5;
const EAGAIN: u64 = 11;
const MSG_PEEK: u64 = 0x2;
const MSG_WAITALL: u64 = 0x100;

/// A socket the host holds for the program.
pub(super) enum Socket {
    Listener(TcpListener),
    Stream(TcpStream),
}

impl SandboxHost {
    /// Pre-open a listening TCP socket on each of `addresses` (`host:port`), as descriptors 3, 4,
    /// ... in order, and set `LISTEN_FDS` for the program. Returns the address each one listens
    /// on, so a port of 0 can be found out.
    pub fn preopen_listeners(&mut self, addresses: &[String]) -> Result<Vec<String>, String> {
        let mut bound = Vec::new();
        for text in addresses {
            let address = text
                .to_socket_addrs()
                .map_err(|e| format!("cannot use the listen address `{text}`: {e}"))?
                .next()
                .ok_or_else(|| format!("the listen address `{text}` names no address"))?;
            let listener = TcpListener::bind(address)
                .map_err(|e| format!("cannot listen on `{text}`: {e}"))?;
            if let Ok(local) = listener.local_addr() {
                bound.push(local.to_string());
            }
            let fd = self.next_socket_fd();
            self.sockets.insert(fd, Socket::Listener(listener));
        }
        self.extra_environment
            .insert("LISTEN_FDS".into(), addresses.len().to_string());
        Ok(bound)
    }

    fn next_socket_fd(&mut self) -> i64 {
        if self.next_fd < super::FIRST_FD {
            self.next_fd = super::FIRST_FD;
        }
        let fd = self.next_fd;
        self.next_fd += 1;
        fd
    }

    /// `accept`, `recv`, `send`, `shutdown`, and `close` on a socket.
    pub(super) fn socket_call(&mut self, fun: LibcFn, arg: impl Fn(usize) -> u64) -> Option<u64> {
        let fd = arg(0) as i32 as i64;
        Some(match fun {
            LibcFn::Close => {
                self.sockets.remove(&fd)?;
                0
            }
            LibcFn::Accept => {
                let accepted = match self.sockets.get(&fd) {
                    Some(Socket::Listener(listener)) => listener.accept(),
                    Some(Socket::Stream(_)) => return Some(self.set_errno(EINVAL)),
                    None => return Some(self.set_errno(self.not_a_socket(fd))),
                };
                match accepted {
                    Ok((stream, _)) => {
                        // WASI does not report the peer's address.
                        if arg(2) != 0 {
                            write_u32(arg(2), 0);
                        }
                        let new = self.next_socket_fd();
                        self.sockets.insert(new, Socket::Stream(stream));
                        new as u64
                    }
                    Err(error) => self.set_errno(errno_of(&error)),
                }
            }
            LibcFn::Recv => {
                // Only PEEK and WAITALL exist in WASI.
                let flags = arg(3) as u32 as u64;
                if flags & !(MSG_PEEK | MSG_WAITALL) != 0 {
                    return Some(self.set_errno(EINVAL));
                }
                let count = arg(2) as usize;
                let stream = match self.sockets.get_mut(&fd) {
                    Some(Socket::Stream(stream)) => stream,
                    Some(Socket::Listener(_)) => return Some(self.set_errno(ENOTCONN)),
                    None => return Some(self.set_errno(self.not_a_socket(fd))),
                };
                let mut buffer = vec![0u8; count];
                let result = if flags & MSG_PEEK != 0 {
                    stream.peek(&mut buffer)
                } else if flags & MSG_WAITALL != 0 {
                    read_all(stream, &mut buffer)
                } else {
                    stream.read(&mut buffer)
                };
                match result {
                    Ok(n) => {
                        write_bytes(arg(1), &buffer[..n]);
                        n as u64
                    }
                    Err(error) => self.set_errno(errno_of(&error)),
                }
            }
            LibcFn::Send => {
                if arg(3) as u32 != 0 {
                    return Some(self.set_errno(EINVAL));
                }
                let bytes = read_bytes(arg(1), arg(2) as usize);
                let stream = match self.sockets.get_mut(&fd) {
                    Some(Socket::Stream(stream)) => stream,
                    Some(Socket::Listener(_)) => return Some(self.set_errno(ENOTCONN)),
                    None => return Some(self.set_errno(self.not_a_socket(fd))),
                };
                match stream.write(&bytes) {
                    Ok(n) => n as u64,
                    Err(error) => self.set_errno(errno_of(&error)),
                }
            }
            LibcFn::Shutdown => {
                // C's SHUT_RD, SHUT_WR and SHUT_RDWR.
                let how = match arg(1) as i32 {
                    0 => Shutdown::Read,
                    1 => Shutdown::Write,
                    2 => Shutdown::Both,
                    _ => return Some(self.set_errno(EINVAL)),
                };
                match self.sockets.get(&fd) {
                    Some(Socket::Stream(stream)) => match stream.shutdown(how) {
                        Ok(()) => 0,
                        Err(error) => self.set_errno(errno_of(&error)),
                    },
                    Some(Socket::Listener(_)) => self.set_errno(ENOTCONN),
                    None => self.set_errno(self.not_a_socket(fd)),
                }
            }
            _ => return None,
        })
    }

    /// `ENOTSOCK` for an open file, `EBADF` for a number that is nothing.
    fn not_a_socket(&self, fd: i64) -> u64 {
        if self.fds.contains_key(&fd) || (0..3).contains(&fd) {
            ENOTSOCK
        } else {
            EBADF
        }
    }
}

/// Read until `buffer` is full or the peer closes, like `recv` with `MSG_WAITALL`.
fn read_all(stream: &mut TcpStream, buffer: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        match stream.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) if filled > 0 => break,
            Err(error) => return Err(error),
        }
    }
    Ok(filled)
}

/// The Linux `errno` for an I/O error.
fn errno_of(error: &std::io::Error) -> u64 {
    use std::io::ErrorKind::{
        BrokenPipe, ConnectionAborted, ConnectionReset, InvalidInput, NotConnected, WouldBlock,
    };
    match error.kind() {
        BrokenPipe => EPIPE,
        ConnectionReset | ConnectionAborted => ECONNRESET,
        NotConnected => ENOTCONN,
        WouldBlock => EAGAIN,
        InvalidInput => EINVAL,
        _ => EIO,
    }
}
