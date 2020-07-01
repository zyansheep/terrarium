
#![allow(dead_code)]
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
		self.read(&mut buf);
		Ok(String::from_utf8(buf).unwrap())
    }
}