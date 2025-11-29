use std::{
    // fs::File,
    io::{self, Read, Result as IoResult, Write}
};

use rack::*;
const HEAD: [u8; 4] = [0x1f, 0xad, 0xa7, 0x24];

trait ReadAll: Read {
    fn read_all(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let mut x = 0;
        while x < buf.len() {
            let n = self.read(&mut buf[x..])?;
            if n == 0 { break; }
            x += n;
        }
        Ok(x)
    }
}

impl<T: Read> ReadAll for T {}

// fn unrack(mut fname: String) -> IoResult<()> {
//     if !fname.ends_with(".rk") {
//         fname = format!("{}.rk", fname);
//     }
//     let mut file = File::open(&fname)?;
//     let mut file_rk = File::create(fname.trim_end_matches(".rk"))?;
//     let mut unrack = Unrack::new();
//     let mut buf = vec![0; 65536];

//     if {
//         let mut head = [0u8; 4];
//         file.read_all(&mut head)?;
//         head != HEAD
//     } {
//         eprintln!("File {} is not a valid rack file", &fname);
//         return Ok(());
//     }
//     while let Ok(n) = file_rk.read_all(&mut buf) {
//         if n == 0 { break; }
//         file.write_all(&unrack.proc(&buf[..n]))?;
//     }
//     return Ok(());
// }

fn unrack_stdio() -> IoResult<()> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut unrack = Unrack::new();
    let mut buf = vec![0; 65536];

    if {
        let mut head = [0u8; 4];
        stdin.read_all(&mut head)?;
        head != HEAD
    } {
        eprintln!("Stream is not a valid rack file");
        return Ok(());
    }
    loop {
        match stdin.read_all(&mut buf) {
            Ok(0) => break,
            Ok(n) => stdout.write_all(&unrack.proc(&buf[..n]))?,
            Err(_) => break,
        }
    }
    return Ok(());
}

fn main() {
    unrack_stdio().unwrap();
    // let mut args = std::env::args();
    // if args.len() < 2 {
    //     unrack_stdio().unwrap();
    // } else {
    //     args.next();
    //     for fname in args {
    //         unrack(fname).unwrap();
    //     }
    // }
}
