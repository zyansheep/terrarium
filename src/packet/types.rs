use std::io;
use std::convert::TryInto;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use variant_encoding::{VarStringReader, VarStringWriter};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum NetworkTextMode {
	Literal = 0u8, Formattable = 1, LocalizationKey = 2,
}
#[derive(Debug, PartialEq)]
pub struct NetworkText {
	mode: NetworkTextMode,
	text: String,
	substitution: Vec<NetworkText>,
}
impl NetworkText {
	pub fn new(string: &str) -> Self {
		NetworkText {
			mode: NetworkTextMode::Literal,
			text: string.to_owned(),
			substitution: vec![],
		}
	}
	pub fn write(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
		writer.write_u8(self.mode as u8)?;
		writer.write_varstring(&self.text)?;
		if self.mode != NetworkTextMode::Literal {
			let num: u8 = self.substitution.len().try_into().unwrap_or(255); //Impossible for this to get too large
			writer.write_u8(num)?;
			for text in self.substitution.iter() {
				text.write(writer)?;
			}
		}
		Ok(())
	}
}