use std::io::{Write, Read, Seek, SeekFrom};

pub struct Palette {
    colors: Box<[(u8, u8, u8)]>
}


