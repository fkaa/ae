use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

// sub_561cf0
use std::io::{Write, Read, Seek, SeekFrom};

pub mod color;
pub mod terrain;

pub use self::color::*;
pub use self::terrain::*;

#[derive(Debug)]
pub struct GameDataFile {
    terrain: Vec<TerrainRestriction>,
    colors: Vec<PlayerColor>
}

impl GameDataFile {
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, ::std::io::Error> {
        reader.seek(SeekFrom::Current(8));

        let restriction_count = try!(reader.read_u16::<LittleEndian>());
        let terrain_count = try!(reader.read_u16::<LittleEndian>());
        reader.seek(SeekFrom::Current((restriction_count * 8) as _));

        let mut terrain = Vec::new();
        println!("{}", restriction_count);
        for _ in 0..restriction_count {
            terrain.push(try!(TerrainRestriction::read(terrain_count as _, reader)));
        }
        //let terrain = try!(TerrainRestriction::read(reader));

        let count = try!(reader.read_u16::<LittleEndian>());
        let mut colors = Vec::new();
        for _ in 0..count {
            colors.push(try!(PlayerColor::read(reader)));
        }

        Ok(GameDataFile {
            terrain: terrain,
            colors: colors
        })
    }
}
