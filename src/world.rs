extern crate byteorder;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io;

#[derive(Default)]
pub struct World {
    version: i32,
    revision: u32,
    is_favorite: bool,
}

impl World {
    pub fn read_from_file(path: &str) -> Result<World, io::Error> {
        let mut file = File::open(path)?;
        let wld = World::read(&mut file)?;

        Ok(wld)
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), io::Error> {
        let mut file = File::create(path)?;
        self.write(&mut file)?;

        Ok(())
    }

    pub fn read(rdr: &mut impl io::Read) -> Result<World, io::Error> {
        let mut wld = World::default();

        wld.read_file_format_header(rdr)?;

        Ok(wld)
    }

    fn read_file_format_header(&mut self, rdr: &mut impl io::Read) -> Result<(), io::Error> {
        self.version = rdr.read_i32::<LittleEndian>()?;
        rdr.read_u64::<LittleEndian>()?; // Magic + filetype (we are assuming that it is a world file.)
        self.revision = rdr.read_u32::<LittleEndian>()?;
        self.is_favorite = rdr.read_u64::<LittleEndian>()? != 0;

        Ok(())
    }

    pub fn write(&self, wtr: &mut impl std::io::Write) -> Result<(), io::Error> {
        self.write_file_format_header(wtr)?;

        Ok(())
    }

    fn write_file_format_header(&self, wtr: &mut impl io::Write) -> Result<(), io::Error> {
        wtr.write_i32::<LittleEndian>(self.version)?;
        wtr.write(b"relogic")?; // Magic.
        wtr.write_u8(2)?; // Filetype.
        wtr.write_u32::<LittleEndian>(self.revision)?;
        wtr.write_u64::<LittleEndian>(self.is_favorite as u64)?;

        Ok(())
    }
}
