use std::{
    fs::File,
    io::{self, Read, Result as IoResult, Write}
};

use rack::*;
const HEAD: [u8; 4] = [0x1f, 0xad, 0xa7, 0x24];

fn rack(fname: String) -> IoResult<()> {
    let mut file = File::open(&fname)?;
    let mut file_rk = File::create(format!("{}.rk", &fname))?;
    let mut rack = Rack::new();
    let mut buf = vec![0; 65536];

    file_rk.write_all(&HEAD)?;
    while let Ok(n) = file.read(&mut buf) {
        if n == 0 { break; }
        file_rk.write_all(&rack.proc(&buf[..n]))?;
    }
    file_rk.write_all(&rack.finish())?;

    return Ok(());
}

fn rack_stdio() -> IoResult<()> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut rack = Rack::new();
    let mut buf = vec![0; 65536];

    stdout.write_all(&HEAD)?;
    loop {
        match stdin.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => stdout.write_all(&rack.proc(&buf[..n]))?,
            Err(_) => break,
        }
    }
    stdout.write_all(&rack.finish())?;
    return Ok(());
}

fn main() {
    let mut args = std::env::args();
    if args.len() < 2 {
        rack_stdio().unwrap();
    } else {
        args.next();
        for fname in args {
            rack(fname).unwrap();
        }
    }
}
