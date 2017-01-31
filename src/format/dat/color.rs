use std::io::{Write, Read, Seek, SeekFrom};

#[repr(C, packed)]
#[derive(Debug)]
pub struct PlayerColor {
    id: i32,
    palette: i32,
    color: i32,
    u1: i32,
    u2: i32,
    minimap_color: u32,
    u3: i32,
    u4: i32,
    stats_color: u32
}

impl PlayerColor {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ::std::io::Error> {
        let mut col: PlayerColor = unsafe { ::std::mem::zeroed() };

        unsafe {
            let col_slice = ::std::slice::from_raw_parts_mut(
                &mut col as *mut _ as *mut u8,
                ::std::mem::size_of::<Self>()
            );

            try!(reader.read_exact(col_slice));
        }

        Ok(col)
    }
}
