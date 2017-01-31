use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

use std::io::{Read, Write, Seek, SeekFrom, Cursor};
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum DrsFile {
    Wav(Box<[u8]>),
    Bin(Box<[u8]>),
    Slp(Box<[u8]>),
    Unknown(u32, Box<[u8]>)
}

#[derive(Debug)]
pub enum DrsError {
    Io(::std::io::Error),
    TableNotFound(i32),
    EntryNotFound(i32)
}

impl From<::std::io::Error> for DrsError {
    fn from(err: ::std::io::Error) -> Self {
        DrsError::Io(err)
    }
}

impl fmt::Display for DrsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DrsError::Io(ref err) => write!(f, "IO error: {}", err),
            DrsError::TableNotFound(ref table) => write!(f, "Table {} not found", table),
            DrsError::EntryNotFound(ref entry) => write!(f, "Entry {} not found", entry)
        }
    }
}

impl error::Error for DrsError {
    fn description(&self) -> &str {
        match *self {
            DrsError::Io(ref err) => err.description(),
            DrsError::TableNotFound(ref table) => "DRS table not found",
            DrsError::EntryNotFound(ref entry) => "DRS entry not found"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DrsError::Io(ref err) => err.cause(),
            _ => None
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct Header {
    copyright: [[u8; 20]; 2],
    version: [u8; 4],
    ty: [u8; 12],
    tables: u32,
    offset: u32
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct TableInfo {
    ext: u32,
    offset: u32,
    size: u32
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct TableEntry {
    id: u32,
    offset: u32,
    size: u32
}

pub struct DrsReader<R: Read + Seek> {
    reader: R,
    tables: (Vec<TableInfo>, Vec<Vec<TableEntry>>)
}

impl<R: Read + Seek> DrsReader<R> {
    pub fn new(mut reader: R) -> Result<DrsReader<R>, DrsError> {
        let header = try!(read_drs_header(&mut reader).map_err(DrsError::Io));
        //println!("{:#?}", header);
        let table_infos = try!(read_drs_table_info(header, &mut reader).map_err(DrsError::Io));
        //println!("{:#?}", table_infos);
        let tables = try!(read_drs_tables(&table_infos, &mut reader).map_err(DrsError::Io));

        /*for table in tables.iter() {
            println!("{}", "Table");
            for entry in table.iter() {
                println!("\tEntry #{} ({})", entry.id, entry.size);
            }
        }*/
        //println!("{:#?}", tables);

        Ok(DrsReader {
            reader: reader,
            tables: (table_infos, tables)
        })
    }

    pub fn read_file(&mut self, table: usize, id: usize) -> Result<DrsFile, DrsError> {
        let ext = try!(self.tables.0.get(table).ok_or(DrsError::TableNotFound(table as _))).ext;

        if let Ok(key) = self.tables.1[table].binary_search_by_key(&id, |&e| e.id as _) {
            let entry = self.tables.1[table][key];

            try!(self.reader.seek(SeekFrom::Start(entry.offset as _)));
            let mut data = {
                let mut vec = Vec::with_capacity(entry.size as _);
                unsafe {
                    vec.set_len(entry.size as _);
                }
                vec.into_boxed_slice()
            };
            try!(self.reader.read_exact(&mut data));

            Ok(match (ext & 0xffffff00) {
                0x62696e00 => DrsFile::Bin(data),
                0x736c7000 => DrsFile::Slp(data),
                0x77617600 => DrsFile::Wav(data),
                _ => DrsFile::Unknown(ext, data)
            })
        } else {
            Err(DrsError::EntryNotFound(id as _))
        }
    }
}

fn read_drs_header<R: Read>(reader: &mut R) -> Result<Header, ::std::io::Error> {
    let mut copyright: [u8; 40] = [0u8; 40];
    let mut version: [u8; 4] = [0u8; 4];
    let mut ty: [u8; 12] = [0u8; 12];

    try!(reader.read_exact(&mut copyright));
    try!(reader.read_exact(&mut version));
    try!(reader.read_exact(&mut ty));

    let tables = try!(reader.read_u32::<LittleEndian>());
    let offset = try!(reader.read_u32::<LittleEndian>());

    Ok(Header {
        copyright: unsafe { ::std::mem::transmute(copyright) },
        version: version,
        ty: ty,
        tables: tables,
        offset: offset
    })
}

fn read_drs_table_info<R: Read + Seek>(header: Header, reader: &mut R) -> Result<Vec<TableInfo>, ::std::io::Error> {
    let mut infos = vec![];
    for x in 0..header.tables {
        let ext = try!(reader.read_u32::<LittleEndian>());
        let offset = try!(reader.read_u32::<LittleEndian>());
        let size = try!(reader.read_u32::<LittleEndian>());

        infos.push(TableInfo {
            ext: ext,
            offset: offset,
            size: size
        });
    }

    Ok(infos)
}

fn read_drs_tables<R: Read + Seek>(table_infos: &Vec<TableInfo>, reader: &mut R) -> Result<Vec<Vec<TableEntry>>, ::std::io::Error> {
    let mut table_entries = vec![];

    for info in table_infos {
        let mut entries = vec![];
        reader.seek(SeekFrom::Start(info.offset as _));

        for _ in 0..info.size {
            let id = try!(reader.read_u32::<LittleEndian>());
            let offset = try!(reader.read_u32::<LittleEndian>());
            let size = try!(reader.read_u32::<LittleEndian>());

            entries.push(TableEntry {
                id: id,
                offset: offset,
                size: size
            });
        }

        table_entries.push(entries);
    }

    Ok(table_entries)
}
