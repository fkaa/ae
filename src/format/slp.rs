use std::io::{Write, Read, Seek, SeekFrom, Cursor};
use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

use format::pal::Palette;

#[repr(C)]
#[derive(Debug)]
pub struct SlpHeader {
    version: u32,
    count: u32,
    comment: [u8; 24]
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SlpEntry {
    data_offset: u32,
    data_end: u32,
    outline_offset: u32,
    palette_offset: u32,
    properties: u32,
    width: i32,
    height: i32,
    hotspot_x: i32,
    hotspot_y: i32
}

pub trait SlpRasterizer {
    fn rasterize<W: Write + Seek>(writer: &mut W, obj: &SlpObject)-> Result<(), ::std::io::Error>;
}

pub struct SlpDefaultRasterizer {
}

impl SlpRasterizer for SlpDefaultRasterizer {
    fn rasterize<W: Write + Seek>(writer: &mut W, obj: &SlpObject) -> Result<(), ::std::io::Error> {
        #[inline]
        fn get_length<R: Read>(op: u8, reader: &mut R) -> u8 {
            let len = op >> 4;
            if len == 0u8 {
                reader.read_u8().unwrap()
            } else {
                len
            }
        }

        #[inline]
        fn draw_bytes<W: Write + Seek, R: Read>(writer: &mut W, reader: &mut R, count: u32) {
            for _ in 0..count {
                let res = writer.write(&[reader.read_u8().unwrap()]);
            }
        }

        #[inline]
        fn draw_byte<W: Write + Seek>(writer: &mut W, byte: u8, count: u8) {
            for _ in 0..count {
                writer.write(&[byte]);
            }
        }

        let mut reader = Cursor::new(&obj.instructions);
        let mut y = 0;

        writer.seek(SeekFrom::Current(obj.skips[0].0 as _));

        while let Ok(inst) = reader.read_u8() {
            let command = inst & 0x0f;

            match command {
                0 | 4 | 8 | 0x0c => {
                    draw_bytes(writer, &mut reader, (inst >> 2) as _);
                },
                1 | 5 | 9 | 0x0d => {
                    writer.seek(SeekFrom::Current((inst >> 2) as _));
                },
                2 => {
                    let len = ((inst as u32 & 0xf0) << 4) + reader.read_u8().unwrap() as u32;
                    draw_bytes(writer, &mut reader, len);
                },
                3 => {
                    let len = ((inst as u32 & 0xf0) << 4) + reader.read_u8().unwrap() as u32;
                    writer.seek(SeekFrom::Current(len as _));
                },
                6 => {
                    let len = get_length(inst, &mut reader);
                    draw_bytes(writer, &mut reader, len as _);
                },
                7 => {
                    let len = get_length(inst, &mut reader);
                    let col = reader.read_u8().unwrap();

                    draw_byte(writer, col, len);
                },
                0x0a => {
                    let len = get_length(inst, &mut reader);
                    let col = reader.read_u8().unwrap();
                    draw_byte(writer, col, len);
                },
                0x0b => {
                    let len = get_length(inst, &mut reader);
                    writer.seek(SeekFrom::Current(len as _));
                },
                0x0e => {
                    match inst {
                        0x0e | 0x1e => {
                        },
                        0x2e | 0x3e => {
                        },
                        0x4e | 0x6e => {
                            draw_byte(writer, 0xff, 1);
                        },
                        0x5e | 0x7e => {
                            let len = reader.read_u8().unwrap();
                            draw_byte(writer, 0xff, len);
                        },
                        _ => {
                            println!("unknown extended 0x{:X}", inst);
                        }
                    }

                },
                0x0f => {
                    //println!("End of line ({})", y);

                    if y < (obj.height - 1) as _ {
                        writer.seek(SeekFrom::Current((obj.skips[y].1 + obj.skips[y + 1].0) as _));
                    } else {
                        writer.seek(SeekFrom::Current((obj.skips[y].1) as _ ));
                    }
                    y += 1;
                }
                _ => {
                    println!("\tUnknown: {}", command);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct SlpObject {
    instructions: Box<[u8]>,
    offsets: Vec<u32>,
    skips: Box<[(u16, u16)]>,
    pub width: i32,
    pub height: i32,
    hotspot: (i32, i32)
}

pub struct SlpReader<R: Read + Seek> {
    reader: R,
    entries: Vec<SlpEntry>
}

impl<R: Read + Seek> SlpReader<R> {
    pub fn new(mut reader: R) -> Result<SlpReader<R>, ::std::io::Error> {
        let header = try!(read_slp_header(&mut reader));
        //println!("{:?}", header);
        let entries = try!(read_slp_entries(header, &mut reader));
        //println!("{:?}", entries);

        Ok(SlpReader {
            reader: reader,
            entries: entries
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn read_shape(&mut self, index: usize) -> Result<SlpObject, ::std::io::Error> {
        let entry = self.entries[index];

        try!(self.reader.seek(SeekFrom::Start(entry.outline_offset as _)));

        let mut skips = Vec::new();
        for _ in 0..entry.height {
            let left = try!(self.reader.read_u16::<LittleEndian>());
            let right = try!(self.reader.read_u16::<LittleEndian>());

            skips.push((left, right));
        }

        let mut offsets = Vec::new();
        for _ in 0..entry.height {
            offsets.push(try!(self.reader.read_u32::<LittleEndian>()));
        }

        println!("{:?}..{:?}", offsets[0], offsets[offsets.len() - 1]);
        println!("{:?}..{:?}", offsets[0], entry.data_end);

        let mut instructions = {
            let mut vec = Vec::with_capacity((entry.data_end - offsets[0]) as _);
            unsafe {
                vec.set_len((entry.data_end - offsets[0]) as _);
            }
            try!(self.reader.read_exact(&mut vec));
            vec.into_boxed_slice()
        };

        Ok(SlpObject {
            instructions: instructions,
            offsets: offsets,
            skips: skips.into_boxed_slice(),
            width: entry.width,
            height: entry.height,
            hotspot: (entry.hotspot_x, entry.hotspot_y)
        })
    }
}


fn read_slp_header<R: Read>(reader: &mut R) -> Result<SlpHeader, ::std::io::Error> {
    let version = try!(reader.read_u32::<LittleEndian>());
    let count = try!(reader.read_u32::<LittleEndian>());

    let mut comment: [u8; 24] = [0u8; 24];
    try!(reader.read_exact(&mut comment));

    Ok(SlpHeader {
        version: version,
        count: count,
        comment: comment,
    })
}

fn read_slp_entries<R: Read>(header: SlpHeader, reader: &mut R) -> Result<Vec<SlpEntry>, ::std::io::Error> {
    let mut entries: Vec<SlpEntry> = vec![];

    for i in 0..header.count {
        let data_offset = try!(reader.read_u32::<LittleEndian>());
        let outline_offset = try!(reader.read_u32::<LittleEndian>());
        let palette_offset = try!(reader.read_u32::<LittleEndian>());
        let properties = try!(reader.read_u32::<LittleEndian>());
        let width = try!(reader.read_i32::<LittleEndian>());
        let height = try!(reader.read_i32::<LittleEndian>());
        let hotspot_x = try!(reader.read_i32::<LittleEndian>());
        let hotspot_y = try!(reader.read_i32::<LittleEndian>());

        if let Some(entry) = entries.last_mut() {
            entry.data_end = outline_offset;
        }

        entries.push(SlpEntry {
            data_offset: data_offset,
            data_end: if i == header.count {
                1
            } else {
                0
            },
            outline_offset: outline_offset,
            palette_offset: palette_offset,
            properties: properties,
            width: width,
            height: height,
            hotspot_x: hotspot_x,
            hotspot_y: hotspot_y
        });
    }

    Ok(entries)
}
