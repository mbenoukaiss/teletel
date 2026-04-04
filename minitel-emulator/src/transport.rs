use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

pub const EMULATOR_PORT: u16 = 3615;

pub struct TcpTransport {
    listener: TcpListener,
    client: Option<TcpStream>,
}

impl TcpTransport {
    pub fn bind() -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", EMULATOR_PORT))?;
        listener.set_nonblocking(true)?;

        Ok(Self {
            listener,
            client: None,
        })
    }

    pub fn read_available(&mut self) -> io::Result<Option<Vec<u8>>> {
        if self.client.is_none() {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(true)?;
                    self.client = Some(stream);
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                Err(err) => return Err(err),
            }
        }

        let client = self.client.as_mut().unwrap();
        let mut buffer = [0u8; 1024];
        match client.read(&mut buffer) {
            Ok(0) => {
                self.client = None;
                Ok(None)
            }
            Ok(n) => Ok(Some(buffer[..n].to_vec())),
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(err) => {
                self.client = None;
                Err(err)
            }
        }
    }

    pub fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        if let Some(client) = self.client.as_mut() {
            client.write_all(bytes)?;
            client.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_and_connect() {
        let transport = TcpTransport::bind();
        assert!(transport.is_ok());
    }
}
