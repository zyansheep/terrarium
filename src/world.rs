#![allow(dead_code)]
extern crate byteorder;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io;

#[derive(Default)]
pub struct World {
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
        rdr.read_i32::<LittleEndian>()?; // Version (we are assuming that it is 230.)

        // File metadata.
        rdr.read_u64::<LittleEndian>()?; // Magic + filetype (we are assuming that it is a world file.)
        self.revision = rdr.read_u32::<LittleEndian>()?;
        self.is_favorite = rdr.read_u64::<LittleEndian>()? != 0;

        // Chunk offsets.
        let chunk_count = rdr.read_i16::<LittleEndian>()?;
        let mut chunk_offsets = vec![0; chunk_count as usize];
        for i in 0..chunk_count {
            chunk_offsets[i as usize] = rdr.read_i32::<LittleEndian>()?;
        }

        Ok(())
    }

    pub fn write(&self, wtr: &mut (impl io::Write + io::Seek)) -> Result<(), io::Error> {
        self.write_file_format_header(wtr)?;

        Ok(())
    }

    fn write_file_format_header(&self, wtr: &mut (impl io::Write + io::Seek)) -> Result<(), io::Error> {
        wtr.write_i32::<LittleEndian>(230)?; // Version.

        // File metadata.
        wtr.write(b"relogic")?; // Magic.
        wtr.write_u8(2)?; // Filetype.
        wtr.write_u32::<LittleEndian>(self.revision)?;
        wtr.write_u64::<LittleEndian>(self.is_favorite as u64)?;

        // Chunk offsets.
        wtr.write_i16::<LittleEndian>(11)?; // Chunk count.
        for _ in 0..11 {
            wtr.write_i32::<LittleEndian>(0)?; // Placeholders for after writing all chunks.
        }

        Ok(())
    }
}
