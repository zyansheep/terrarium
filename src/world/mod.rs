extern crate byteorder;
mod utils;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io;
use utils::{WorldReader, WorldWriter};

#[derive(Default)]
pub struct World {
    revision: u32,
    is_favorite: bool,
    tile_frame_important: Vec<bool>,
    name: String,
}

impl World {
    pub fn read_from_file(path: &str) -> Result<World, io::Error> {
        let file = File::open(path)?;
        let mut reader = io::BufReader::new(file);
        let wld = World::read(&mut reader)?;

        Ok(wld)
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), io::Error> {
        let file = File::create(path)?;
        let mut writer = io::BufWriter::new(file);
        self.write(&mut writer)?;

        Ok(())
    }

    pub fn read(rdr: &mut impl io::BufRead) -> Result<World, io::Error> {
        let mut wld = World::default();

        wld.read_file_format_header(rdr)?;
        wld.read_world_header(rdr)?;

        Ok(wld)
    }

    fn read_file_format_header(&mut self, rdr: &mut impl io::BufRead) -> Result<(), io::Error> {
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
        println!("{:#?}", chunk_offsets);

        // Tile frame important.
        let tile_count = rdr.read_i16::<LittleEndian>()?;
        self.tile_frame_important = vec!(false; tile_count as usize);

        let mut byte = rdr.read_u8()?;
        let mut bit = 0;
        for i in 0..tile_count {
            if byte & (1 << bit) != 0 {
                self.tile_frame_important[i as usize] = true;
            }
            bit += 1;
            if bit == 8 {
                byte = rdr.read_u8()?;
                bit = 0;
            }
        }

        Ok(())
    }

    fn read_world_header(&mut self, reader: &mut impl io::BufRead) -> Result<(), io::Error> {
        self.name = reader.read_varint_string()?;
        println!("{:?}", self.name);

        Ok(())
    }


    pub fn write(&self, wtr: &mut (impl io::Write + io::Seek)) -> Result<(), io::Error> {
        self.write_file_format_header(wtr)?;
        self.write_world_header(wtr)?;

        Ok(())
    }

    fn write_file_format_header(&self, wtr: &mut (impl io::Write + io::Seek)) -> Result<(), io::Error> {
        wtr.write_i32::<LittleEndian>(230)?; // World File Version

        // File metadata
        wtr.write(b"relogic")?; // Magic letters
        wtr.write_u8(2)?; // Filetype
        wtr.write_u32::<LittleEndian>(self.revision)?;
        wtr.write_u64::<LittleEndian>(self.is_favorite as u64)?;

        // Chunk offsets
        wtr.write_i16::<LittleEndian>(11)?; // Chunk count
        for _ in 0..11 {
            wtr.write_i32::<LittleEndian>(0)?; // Placeholders for after writing all chunks
        }

        // Tile frame important
        wtr.write_i16::<LittleEndian>(self.tile_frame_important.len() as i16)?;

        let mut byte = 0;
        let mut bit = 0;
        for i in 0..self.tile_frame_important.len() {
            if self.tile_frame_important[i] {
                byte |= 1 << bit;
            }
            bit += 1;
            if bit == 8 {
                wtr.write_u8(byte)?;
                byte = 0;
                bit = 0;
            }
        }
        if bit != 0 {
            wtr.write_u8(byte)?;
        }

        Ok(())
    }
    fn write_world_header(&self, writer: &mut (impl io::Write + io::Seek)) -> io::Result<()> {
        writer.write_varint_string(&self.name)?;

        Ok(())
    }
}
