use std::{
    fs::{File, FileTimes, remove_file},
    io::{self, Read, Result as IoResult, Write}, process::ExitCode
};

use rack::*;
const HEAD: [u8; 4] = [0x1f, 0xad, 0xa7, 0x24];

fn unrack(mut fname: String) -> IoResult<()> {
    if !fname.ends_with(".rk") {
        fname = format!("{}.rk", fname);
    }
    let mut file_rk = File::open(&fname)?;
    let mut file_unrk: File = File::create(fname.trim_end_matches(".rk"))?;
    let mut unrack = Unrack::new();
    let mut buf = vec![0; 65536];

    let res = || -> IoResult<()> {
        let fmeta = file_rk.metadata()?;
        if {
            let mut head = [0u8; HEAD.len()];
            file_rk.read_exact(&mut head)?;
            head != HEAD
        } {
            eprintln!("File {} is not a valid rack file", &fname);
            return Ok(());
        }
        while let Ok(n) = file_rk.read(&mut buf) {
            if n == 0 { break; }
            file_unrk.write_all(&unrack.proc(&buf[..n]))?;
        }

        file_unrk.set_permissions(
            fmeta.permissions()
        )?;
        file_unrk.set_times(
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
        if let Err(e) = remove_file(fname.trim_end_matches(".rk")) {
            eprintln!("rm {} failed: {}", fname.trim_end_matches(".rk"), e);
        }
    }

    return res;
}

fn unrack_stdio() -> IoResult<()> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut unrack = Unrack::new();
    let mut buf = vec![0; 65536];

    if {
        let mut head = [0u8; HEAD.len()];
        stdin.read(&mut head)?;
        head != HEAD
    } {
        eprintln!("Stream is not a valid rack file");
        return Ok(());
    }
    loop {
        match stdin.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => stdout.write_all(&unrack.proc(&buf[..n]))?,
            Err(_) => break,
        }
    }
    return Ok(());
}

fn main() -> ExitCode {
    let mut args = std::env::args();
    let mut err = 0;
    if args.len() < 2 {
        unrack_stdio().unwrap();
    } else {
        let exec = args.next().unwrap();
        for fname in args {
            if let Err(e) = unrack(fname.clone()) {
                eprintln!("{}: {} failed - {}", &exec, &fname, e);
                err += 1;
            }
        }
    }

    return ExitCode::from(err as u8);
}
