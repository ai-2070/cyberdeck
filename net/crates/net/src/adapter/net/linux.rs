//! Linux-specific optimizations for Net.
//!
//! This module provides:
//! - sendmmsg/recvmmsg for batched I/O
//! - io_uring support (optional, requires `net-uring` feature)
//! - Socket configuration for high-throughput

use bytes::{Bytes, BytesMut};
use std::io;
use std::net::SocketAddr;
use std::os::unix::io::RawFd;

use super::protocol::MAX_PACKET_SIZE;

/// Maximum number of messages in a single sendmmsg/recvmmsg call
pub const MAX_BATCH_SIZE: usize = 64;

/// Batched transport using sendmmsg/recvmmsg.
///
/// This provides significantly higher throughput than individual
/// send/recv calls by amortizing syscall overhead.
pub struct BatchedTransport {
    /// Socket file descriptor
    socket_fd: RawFd,
    /// Pre-allocated iovec structures
    iovecs: Vec<libc::iovec>,
    /// Pre-allocated mmsghdr structures
    msgs: Vec<libc::mmsghdr>,
    /// Pre-allocated sockaddr_in structures
    addrs: Vec<libc::sockaddr_in>,
    /// Receive buffers
    recv_buffers: Vec<BytesMut>,
}

impl BatchedTransport {
    /// Create a new batched transport from a socket file descriptor.
    pub fn new(socket_fd: RawFd) -> Self {
        let mut iovecs = Vec::with_capacity(MAX_BATCH_SIZE);
        let mut msgs = Vec::with_capacity(MAX_BATCH_SIZE);
        let mut addrs = Vec::with_capacity(MAX_BATCH_SIZE);
        let mut recv_buffers = Vec::with_capacity(MAX_BATCH_SIZE);

        for _ in 0..MAX_BATCH_SIZE {
            iovecs.push(libc::iovec {
                iov_base: std::ptr::null_mut(),
                iov_len: 0,
            });

            addrs.push(unsafe { std::mem::zeroed() });

            msgs.push(libc::mmsghdr {
                msg_hdr: libc::msghdr {
                    msg_name: std::ptr::null_mut(),
                    msg_namelen: 0,
                    msg_iov: std::ptr::null_mut(),
                    msg_iovlen: 0,
                    msg_control: std::ptr::null_mut(),
                    msg_controllen: 0,
                    msg_flags: 0,
                },
                msg_len: 0,
            });

            recv_buffers.push(BytesMut::with_capacity(MAX_PACKET_SIZE));
        }

        Self {
            socket_fd,
            iovecs,
            msgs,
            addrs,
            recv_buffers,
        }
    }

    /// Send multiple packets in a single syscall.
    ///
    /// Returns the number of packets successfully sent.
    pub fn send_batch(&mut self, packets: &[Bytes], target: SocketAddr) -> io::Result<usize> {
        let count = packets.len().min(MAX_BATCH_SIZE);
        if count == 0 {
            return Ok(0);
        }

        // Convert target address
        let target_addr = match target {
            SocketAddr::V4(addr) => {
                let mut sockaddr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
                sockaddr.sin_family = libc::AF_INET as u16;
                sockaddr.sin_port = addr.port().to_be();
                sockaddr.sin_addr.s_addr = u32::from_ne_bytes(addr.ip().octets());
                sockaddr
            }
            SocketAddr::V6(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "IPv6 not yet supported for batched I/O",
                ));
            }
        };

        // Setup messages
        for (i, packet) in packets.iter().take(count).enumerate() {
            self.iovecs[i] = libc::iovec {
                iov_base: packet.as_ptr() as *mut _,
                iov_len: packet.len(),
            };

            self.addrs[i] = target_addr;

            self.msgs[i].msg_hdr = libc::msghdr {
                msg_name: &mut self.addrs[i] as *mut _ as *mut _,
                msg_namelen: std::mem::size_of::<libc::sockaddr_in>() as u32,
                msg_iov: &mut self.iovecs[i],
                msg_iovlen: 1,
                msg_control: std::ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            };
            self.msgs[i].msg_len = 0;
        }

        // Send
        let sent = unsafe {
            libc::sendmmsg(
                self.socket_fd,
                self.msgs.as_mut_ptr(),
                count as u32,
                0, // flags
            )
        };

        if sent < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(sent as usize)
        }
    }

    /// Receive multiple packets in a single syscall.
    ///
    /// Returns a vector of (data, source_address) tuples.
    pub fn recv_batch(&mut self, max_count: usize) -> io::Result<Vec<(Bytes, SocketAddr)>> {
        let count = max_count.min(MAX_BATCH_SIZE);
        if count == 0 {
            return Ok(Vec::new());
        }

        // Setup receive buffers
        for i in 0..count {
            self.recv_buffers[i].resize(MAX_PACKET_SIZE, 0);

            self.iovecs[i] = libc::iovec {
                iov_base: self.recv_buffers[i].as_mut_ptr() as *mut _,
                iov_len: MAX_PACKET_SIZE,
            };

            self.addrs[i] = unsafe { std::mem::zeroed() };

            self.msgs[i].msg_hdr = libc::msghdr {
                msg_name: &mut self.addrs[i] as *mut _ as *mut _,
                msg_namelen: std::mem::size_of::<libc::sockaddr_in>() as u32,
                msg_iov: &mut self.iovecs[i],
                msg_iovlen: 1,
                msg_control: std::ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            };
            self.msgs[i].msg_len = 0;
        }

        // Receive (non-blocking)
        let received = unsafe {
            libc::recvmmsg(
                self.socket_fd,
                self.msgs.as_mut_ptr(),
                count as u32,
                libc::MSG_DONTWAIT, // Non-blocking
                std::ptr::null_mut(),
            )
        };

        if received < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::WouldBlock {
                return Ok(Vec::new());
            }
            return Err(err);
        }

        // Collect results
        let mut results = Vec::with_capacity(received as usize);
        for i in 0..(received as usize) {
            let len = self.msgs[i].msg_len as usize;
            let mut buffer = std::mem::replace(
                &mut self.recv_buffers[i],
                BytesMut::with_capacity(MAX_PACKET_SIZE),
            );
            buffer.truncate(len);

            let addr = sockaddr_to_socket_addr(&self.addrs[i])?;
            results.push((buffer.freeze(), addr));
        }

        Ok(results)
    }

    /// Receive multiple packets, blocking until at least one is available.
    pub fn recv_batch_blocking(
        &mut self,
        max_count: usize,
    ) -> io::Result<Vec<(Bytes, SocketAddr)>> {
        let count = max_count.min(MAX_BATCH_SIZE);
        if count == 0 {
            return Ok(Vec::new());
        }

        // Setup receive buffers
        for i in 0..count {
            self.recv_buffers[i].resize(MAX_PACKET_SIZE, 0);

            self.iovecs[i] = libc::iovec {
                iov_base: self.recv_buffers[i].as_mut_ptr() as *mut _,
                iov_len: MAX_PACKET_SIZE,
            };

            self.addrs[i] = unsafe { std::mem::zeroed() };

            self.msgs[i].msg_hdr = libc::msghdr {
                msg_name: &mut self.addrs[i] as *mut _ as *mut _,
                msg_namelen: std::mem::size_of::<libc::sockaddr_in>() as u32,
                msg_iov: &mut self.iovecs[i],
                msg_iovlen: 1,
                msg_control: std::ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            };
            self.msgs[i].msg_len = 0;
        }

        // Receive (blocking)
        let received = unsafe {
            libc::recvmmsg(
                self.socket_fd,
                self.msgs.as_mut_ptr(),
                count as u32,
                0, // Blocking
                std::ptr::null_mut(),
            )
        };

        if received < 0 {
            return Err(io::Error::last_os_error());
        }

        // Collect results
        let mut results = Vec::with_capacity(received as usize);
        for i in 0..(received as usize) {
            let len = self.msgs[i].msg_len as usize;
            let mut buffer = std::mem::replace(
                &mut self.recv_buffers[i],
                BytesMut::with_capacity(MAX_PACKET_SIZE),
            );
            buffer.truncate(len);

            let addr = sockaddr_to_socket_addr(&self.addrs[i])?;
            results.push((buffer.freeze(), addr));
        }

        Ok(results)
    }
}

impl std::fmt::Debug for BatchedTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchedTransport")
            .field("socket_fd", &self.socket_fd)
            .field("max_batch_size", &MAX_BATCH_SIZE)
            .finish()
    }
}

/// Convert sockaddr_in to SocketAddr
fn sockaddr_to_socket_addr(addr: &libc::sockaddr_in) -> io::Result<SocketAddr> {
    let ip = std::net::Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr));
    let port = u16::from_be(addr.sin_port);
    Ok(SocketAddr::new(ip.into(), port))
}

/// Configure socket for high-throughput operation.
pub fn configure_socket_for_throughput(fd: RawFd) -> io::Result<()> {
    // Increase buffer sizes
    unsafe {
        let recv_buf: i32 = 64 * 1024 * 1024; // 64 MB
        let send_buf: i32 = 64 * 1024 * 1024; // 64 MB

        if libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            &recv_buf as *const _ as *const libc::c_void,
            std::mem::size_of::<i32>() as u32,
        ) < 0
        {
            return Err(io::Error::last_os_error());
        }

        if libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_SNDBUF,
            &send_buf as *const _ as *const libc::c_void,
            std::mem::size_of::<i32>() as u32,
        ) < 0
        {
            return Err(io::Error::last_os_error());
        }

        // Enable busy polling (reduces latency)
        let busy_poll: i32 = 50; // microseconds
        let _ = libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_BUSY_POLL,
            &busy_poll as *const _ as *const libc::c_void,
            std::mem::size_of::<i32>() as u32,
        );

        // Disable fragmentation
        let pmtu: i32 = libc::IP_PMTUDISC_DO;
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_IP,
            libc::IP_MTU_DISCOVER,
            &pmtu as *const _ as *const libc::c_void,
            std::mem::size_of::<i32>() as u32,
        );
    }

    Ok(())
}

/// Enable nanosecond timestamps on the socket.
#[allow(dead_code)]
pub fn enable_timestamps(fd: RawFd) -> io::Result<()> {
    unsafe {
        let enable: i32 = 1;
        if libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_TIMESTAMPNS,
            &enable as *const _ as *const libc::c_void,
            std::mem::size_of::<i32>() as u32,
        ) < 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    use std::os::unix::io::AsRawFd;

    #[test]
    fn test_batched_transport_creation() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let fd = socket.as_raw_fd();
        let transport = BatchedTransport::new(fd);

        assert!(transport.iovecs.len() == MAX_BATCH_SIZE);
        assert!(transport.msgs.len() == MAX_BATCH_SIZE);
    }

    #[test]
    fn test_send_recv_batch() {
        let socket1 = UdpSocket::bind("127.0.0.1:0").unwrap();
        let socket2 = UdpSocket::bind("127.0.0.1:0").unwrap();

        socket1.set_nonblocking(true).unwrap();
        socket2.set_nonblocking(true).unwrap();

        let addr1 = socket1.local_addr().unwrap();
        let addr2 = socket2.local_addr().unwrap();

        let mut transport1 = BatchedTransport::new(socket1.as_raw_fd());
        let mut transport2 = BatchedTransport::new(socket2.as_raw_fd());

        // Send batch from transport2 to transport1
        let packets = vec![
            Bytes::from_static(b"packet1"),
            Bytes::from_static(b"packet2"),
            Bytes::from_static(b"packet3"),
        ];

        let sent = transport2.send_batch(&packets, addr1).unwrap();
        assert_eq!(sent, 3);

        // Small delay for packets to arrive
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Receive batch on transport1
        let received = transport1.recv_batch(10).unwrap();
        assert_eq!(received.len(), 3);

        assert_eq!(&received[0].0[..], b"packet1");
        assert_eq!(&received[1].0[..], b"packet2");
        assert_eq!(&received[2].0[..], b"packet3");

        for (_, source) in &received {
            assert_eq!(*source, addr2);
        }
    }

    #[test]
    fn test_configure_socket() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let fd = socket.as_raw_fd();

        // Should not fail
        configure_socket_for_throughput(fd).unwrap();
    }
}
