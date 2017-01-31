extern crate inflate;
extern crate byteorder;

mod format;

use format::{drs, slp, dat};
use format::slp::SlpRasterizer;


use std::io::{BufReader, Cursor, Write};
use std::fs::File;

fn datfile() {
    use inflate::InflateStream;
    use std::io::Read;

    let mut data = Vec::new();
    let mut f = File::open("/Applications/Age of Empires II/Data/empires2_x1_p1.dat").unwrap();
    let mut reader = BufReader::new(f);
    reader.read_to_end(&mut data);

    let mut inflater = InflateStream::new();
    let mut out = Vec::<u8>::new();
    let mut n = 0;
    while n < data.len() {
        let res = inflater.update(&data[n..]);
        if let Ok((num_bytes_read, result)) = res {
            n += num_bytes_read;
            out.extend(result);
        } else {
            res.unwrap();
        }
    }

    let dat = dat::GameDataFile::read(&mut Cursor::new(out));

    println!("{:#?}", dat);

//    let mut f = File::create("bar.bin").unwrap();
//    f.write_all(&out);


    //println!("{}", unsafe {::std::str::from_utf8_unchecked(&out) });
}

fn main() {
    datfile();

    let mut f = File::open("/Applications/Age of Empires II/Data/interfac.drs").unwrap();
    let mut reader = BufReader::new(f);

    let mut contents = drs::DrsReader::new(&mut reader).unwrap();
    let file = contents.read_file(0, 53154).unwrap();

    match file {
        drs::DrsFile::Slp(data) => {
            let mut cur = Cursor::new(data);
            let mut slp_reader = slp::SlpReader::new(&mut cur).unwrap();
            let slp = slp_reader.read_shape(0).unwrap();

            let mut buf = vec![0u8; (slp.width * slp.height) as _];
            let mut cur = Cursor::new(buf.into_boxed_slice());
            slp::SlpDefaultRasterizer::rasterize(&mut cur, &slp);

            use std::io::Write;
            let mut f = File::create("foo.bin").unwrap();
            f.write_all(&cur.into_inner());

            println!("{:?}", slp);
        },
        drs::DrsFile::Bin(data) => {

            //println!("{}", unsafe {::std::str::from_utf8_unchecked(&out) });
            use std::io::Write;
            
            let mut f = File::create("foo.bmp").unwrap();
            f.write_all(&data);

        },
        a @ _ => {println!("{:?}", a);}
    }

}
