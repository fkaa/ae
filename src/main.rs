extern crate inflate;
extern crate byteorder;
extern crate image;

mod format;

use format::{drs, slp, dat};
use format::slp::SlpRasterizer;


use std::io::{BufReader, Cursor, Write};
use std::fs::File;
use std::path::Path;

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

fn dump_slp(file: &Path, out: &Path) {
    let mut reader = BufReader::new(File::open(file).unwrap());

    let mut reader = slp::SlpReader::new(&mut reader).unwrap();
    for i in 0..reader.len() {
        let obj = reader.read_shape(i).unwrap();

        let mut buf = vec![0u8; (obj.width * obj.height) as _];
        let mut cur = Cursor::new(buf.into_boxed_slice());
        slp::SlpDefaultRasterizer::rasterize(&mut cur, &obj);

        use image::png::PNGEncoder;
        use image::ColorType;

        let outpath = out.join(Path::new(format!("{}-{}.png", file.file_name().unwrap().to_str().unwrap(), i).as_str()));
        let mut outfile = File::create(outpath).unwrap();
        PNGEncoder::new(outfile).encode(&cur.into_inner(), obj.width as _, obj.height as _, ColorType::Gray(8));

    }
}

// 133 x 306

fn main() {
    dump_slp(Path::new("C:/Program Files (x86)/Steam/steamapps/common/Age2HD/resources/_common/drs/gamedata_x2/6249.slp"), Path::new("./dump/slp/"))
    //datfile();

/*
    let mut f = File::open("/Applications/Age of Empires II/Data/interfac.drs").unwrap();
    let mut reader = BufReader::new(f);

    let mut contents = drs::DrsReader::new(&mut reader).unwrap();
    let (table, idx) = (1, 53154);
    let file = contents.read_file(table, idx).unwrap();

    match file {
        drs::DrsFile::Slp(data) => {
            let mut cur = Cursor::new(data);
            let mut slp_reader = slp::SlpReader::new(&mut cur).unwrap();
            let slp = slp_reader.read_shape(0).unwrap();

            let mut buf = vec![0u8; (slp.width * slp.height) as _];
            let mut cur = Cursor::new(buf.into_boxed_slice());
            slp::SlpDefaultRasterizer::rasterize(&mut cur, &slp);

            use image::png::PNGEncoder;
            use image::ColorType;

            let mut f = File::create(format!("slp-{}-{}.png", table, idx)).unwrap();
            PNGEncoder::new(f).encode(&cur.into_inner(), slp.width as _, slp.height as _, ColorType::Gray(8));

//            println!("{:?}", slp);
        },
        drs::DrsFile::Bin(data) => {

            //println!("{}", unsafe {::std::str::from_utf8_unchecked(&out) });
            use std::io::Write;
            
            let mut f = File::create("foo.bmp").unwrap();
            f.write_all(&data);

        },
        a @ _ => {println!("{:?}", a);}
    }*/

}
