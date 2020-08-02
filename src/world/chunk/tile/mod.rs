use serde::{Serialize, Deserialize};

use std::io;

use byteorder::{WriteBytesExt, ReadBytesExt, LittleEndian, ByteOrder};
pub mod wall;
pub use wall::{Wall, WallType};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct TileCoord {
	pub x: u16,
	pub y: u16,
}
impl TileCoord {
	pub fn write<Order: ByteOrder>(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
		writer.write_u16::<Order>(self.x)?;
		writer.write_u16::<Order>(self.y)?;
		Ok(())
	}
	pub fn read<Order: ByteOrder>(reader: &mut impl io::Read) -> Result<TileCoord, io::Error> {
		Ok(TileCoord {
			x: reader.read_u16::<Order>()?,
			y: reader.read_u16::<Order>()?
		})
	}
}

/// A tile in a World.
#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Tile {
	/// The [Block](struct.Block.html) of the tile.
	pub block: Option<Block>,
	/// The [Wall](struct.Wall.html) of the tile.
	pub wall: Option<Wall>, // 0 if no wall
	/// The [Liquid](struct.Liquid.html) of the tile.
	pub liquid: Option<Liquid>,
	/// True for each of the 4 colors of wire on the tile.
	pub has_wire: [bool; 4],
	/// True if there is an actuator on the tile.
	pub has_actuator: bool,
}
impl Tile {
	pub fn new(block: Block) -> Tile {
		Tile {
			block: Some(block),
			..Tile::default()
		}
	}
}

/// A foreground tile.
#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Block {
	/// The ID of the block.
	pub id: u16,
	/// The color of the block.
	pub color: u8,
	/// The width of the block frame.
	pub frame_width: u16,
	/// The height of the block frame.
	pub frame_height: u16,
	/// The slope of the block.
	pub slope: u8,
	/// True if the block is inactive.
	pub is_inactive: bool,
}
impl Block {
	pub fn new(id: u16) -> Block {
		Block {id, ..Block::default()}
	}
}

/// A liquid tile.
#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Liquid {
	/// The ID of the liquid.
	pub id: u8,
	/// The amount of the liquid.
	pub amount: u8,
}
