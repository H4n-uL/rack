use std::io::{Read, Result};

pub trait ReadAll: Read {
    fn read_all(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut x = 0;
        while x < buf.len() {
            let n = self.read(&mut buf[x..])?;
            if n == 0 { break; }
            x += n;
        }
        Ok(x)
    }
}

impl<T: Read> ReadAll for T {}
