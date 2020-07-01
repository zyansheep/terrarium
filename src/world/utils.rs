use std::io;

extern crate integer_encoding;
use integer_encoding::VarIntReader;

pub trait WorldReader {
	fn read_varint_string(&mut self) -> io::Result<String>;
}
impl<R: io::Read> WorldReader for R {
    fn read_varint_string(&mut self) -> io::Result<String> {
		let length: usize = self.read_varint()?;
		let mut buf = vec![0 as u8; length];
		self.read(&mut buf)?;
		Ok(String::from_utf8(buf).unwrap())
    }
}

use integer_encoding::VarIntWriter;
pub trait WorldWriter {
	fn write_varint_string(&mut self, string: &String) -> io::Result<usize>;
}
impl<W: io::Write> WorldWriter for W {
	fn write_varint_string(&mut self, string: &String) -> io::Result<usize> {
		let mut written = self.write_varint(string.len())?;
		written += self.write(string.as_bytes())?;
		Ok(written)
	}
}