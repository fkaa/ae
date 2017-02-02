use std::io::{Write, BufRead, Read, Seek, SeekFrom};

pub struct Palette {
    colors: Box<[(u8, u8, u8)]>
}

impl Palette {
    pub fn parse<R: BufRead>(data: &mut R) -> Result<Self, ()> {
        let mut lines = data.lines();

        if (lines.next().unwrap().unwrap() != "JASC-PAL") {
            // err
        }

        let mut colors = Vec::new();
        // err


        Ok(Palette {
            colors: colors.into_boxed_slice()
        })
    }
}
