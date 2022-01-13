use std::io::{self, Read, Write};

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

impl Read for StreamMock {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        if self.in_buffer.len() > 0 {
            buf.write_all(self.in_buffer.as_bytes())?;
            Ok(self.in_buffer.len())
        } else {
            Err(io::Error::from(io::ErrorKind::WouldBlock))
        }
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
