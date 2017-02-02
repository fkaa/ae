use std::io::{Write, BufRead, Read, Seek, SeekFrom};

pub struct Palette {
    pub colors: Box<[(u8, u8, u8)]>
}

impl Palette {
    pub fn parse<R: BufRead>(data: &mut R) -> Result<Self, ()> {
        let mut lines = data.lines();

        if lines.next().unwrap().unwrap() != "JASC-PAL" {
            // err
        }

        lines.next().unwrap();
        let len = lines.next().unwrap().unwrap().parse::<i32>().unwrap();
        let mut colors = Vec::new();

        while let Some(Ok(line)) = lines.next() {
            let mut nums = line.split(" ");
            let r = nums.next().unwrap().parse::<u8>().unwrap();
            let g = nums.next().unwrap().parse::<u8>().unwrap();
            let b = nums.next().unwrap().parse::<u8>().unwrap();

            colors.push((r, g, b));
        }

        assert!(colors.len() == len as _);

        Ok(Palette {
            colors: colors.into_boxed_slice()
        })
    }
}
