use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

use std::io::{Write, Read, Seek, SeekFrom};

#[repr(C, packed)]
#[derive(Debug)]
pub struct TerrainPassGraphic {
    exit_tile: i32,
    enter_tile: i32,
    walk_tile: i32,
    walk_rate: i32
}

impl TerrainPassGraphic {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ::std::io::Error> {
        let mut terrain: TerrainPassGraphic = unsafe { ::std::mem::zeroed() };

        unsafe {
            let terrain_slice = ::std::slice::from_raw_parts_mut(
                &mut terrain as *mut _ as *mut u8,
                ::std::mem::size_of::<Self>()
            );

            try!(reader.read_exact(terrain_slice));
        }

        Ok(terrain)
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct TerrainRestriction {
    terrain_accessible: Vec<f32>,
    terrain_pass_graphics: Vec<TerrainPassGraphic>
}

impl TerrainRestriction {
    pub fn read<R: Read + Seek>(count: usize, reader: &mut R) -> Result<Self, ::std::io::Error> {
        let mut accessible = Vec::new();
        for _ in 0..count {
            accessible.push(try!(reader.read_f32::<LittleEndian>()));
        }

        let mut graphics = Vec::new();
        for _ in 0..count {
            graphics.push(try!(TerrainPassGraphic::read(reader)));
        }

        Ok(TerrainRestriction {
            terrain_accessible: accessible,
            terrain_pass_graphics: graphics
        })
    }
}

