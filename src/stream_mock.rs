use core::task::Poll;
use std::io::{Read, Write};

use tokio::io::{AsyncRead, AsyncWrite};

pub struct StreamMock {
    pub in_buffer: String,
    pub out_buffer: String,
}

impl StreamMock {
    pub fn new() -> Self {
        Self {
            in_buffer: String::new(),
            out_buffer: String::new(),
        }
    }

    pub fn receive(&mut self, message: &str) {
        self.in_buffer.push_str(message)
    }
}

impl AsyncRead for StreamMock {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.in_buffer.is_empty() {
            Poll::Pending
        } else {
            buf.put_slice(self.in_buffer.as_bytes());
            Poll::Ready(Ok(()))
        }
    }
}

impl AsyncWrite for StreamMock {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        self.out_buffer
            .push_str(&buf.iter().map(|c| *c as char).collect::<String>());
        println!("{}", self.out_buffer);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl Read for StreamMock {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        buf.write_all(self.in_buffer.as_bytes())?;
        Ok(self.in_buffer.len())
    }
}

impl Write for StreamMock {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.out_buffer
            .push_str(&buf.iter().map(|c| *c as char).collect::<String>());
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
