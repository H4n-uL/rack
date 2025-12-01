use std::{
    fs::{File, FileTimes, remove_file},
    io::{self, Read, Result as IoResult, Write}, process::ExitCode
};

use rack::*;
const HEAD: [u8; 4] = [0x1f, 0xad, 0xa7, 0x24];

fn rack(fname: String) -> IoResult<()> {
    let mut file = File::open(&fname)?;
    let mut file_rk = File::create(format!("{}.rk", &fname))?;
    let mut rack = Rack::new();
    let mut buf = vec![0; 65536];

    let res = || -> IoResult<()> {
        let fmeta = file.metadata()?;

        file_rk.write_all(&HEAD)?;
        loop {
            match file.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => file_rk.write_all(&rack.proc(&buf[..n]))?,
                Err(e) => return Err(e),
            }
        }
        file_rk.write_all(&rack.finish())?;

        file_rk.set_permissions(
            fmeta.permissions()
        )?;
        file_rk.set_times(
            FileTimes::new()
                .set_accessed(fmeta.accessed()?)
                .set_modified(fmeta.modified()?)
        )?;

        return Ok(());
    }();

    if res.is_ok() {
        if let Err(e) = remove_file(&fname) {
            eprintln!("rm {} failed: {}", &fname, e);
        }
    } else {
        if let Err(e) = remove_file(format!("{}.rk", &fname)) {
            eprintln!("rm {}.rk failed: {}", &fname, e);
        }
    }

    return res;
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

fn main() -> ExitCode {
    let mut args = std::env::args();
    let mut err = 0;
    if args.len() < 2 {
        rack_stdio().unwrap();
    } else {
        let exec = args.next().unwrap();
        for fname in args {
            if let Err(e) = rack(fname.clone()) {
                eprintln!("{}: {} failed - {}", &exec, &fname, e);
                err += 1;
            }
        }
    }

    return ExitCode::from(err as u8);
}
