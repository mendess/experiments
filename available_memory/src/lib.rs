use std::{
    cmp::min,
    fs::File,
    io::{self, Read},
};

pub fn ram_streaming<const N: usize>() -> io::Result<usize> {
    #[derive(Debug)]
    enum ParseState {
        Seeking,
        ParsingKey(&'static [u8]),
        SkippingWhitespace,
        ParsingValue(usize),
    }

    const KEY: &[u8] = b"MemAvailable:";

    let mut file = File::open("/proc/meminfo")?;
    let mut buf = [0u8; N];
    let mut state = ParseState::Seeking;
    loop {
        let read = match file.read(&mut buf) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(read) => read,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        let mut buf = &buf[..read];
        while !buf.is_empty() {
            (buf, state) = match state {
                ParseState::Seeking => match buf.iter().position(|&b| b == KEY[0]) {
                    Some(key_start) => (&buf[key_start + 1..], ParseState::ParsingKey(&KEY[1..])),
                    None => (&[][..], ParseState::Seeking),
                },
                ParseState::ParsingKey(missing) => {
                    let len = min(missing.len(), buf.len());
                    let state = if missing[..len] == buf[..len] {
                        match &missing[len..] {
                            [] => ParseState::SkippingWhitespace,
                            missing => ParseState::ParsingKey(missing),
                        }
                    } else {
                        ParseState::Seeking
                    };
                    (&buf[len..], state)
                }
                ParseState::SkippingWhitespace => {
                    let buf = buf.trim_ascii_start();
                    let state = if buf.is_empty() {
                        ParseState::SkippingWhitespace
                    } else {
                        ParseState::ParsingValue(0)
                    };
                    (buf, state)
                }
                ParseState::ParsingValue(n) => {
                    let digit_count = buf
                        .iter()
                        .position(|b| !b.is_ascii_digit())
                        .unwrap_or(buf.len());
                    let digit_buf = &buf[..digit_count];
                    match std::str::from_utf8(digit_buf)
                        .ok()
                        .and_then(|b| b.parse::<usize>().ok())
                    {
                        Some(d) => {
                            let decimal_shift =
                                10usize.pow(digit_count.try_into().expect("should fit in u32"));
                            (
                                &buf[digit_buf.len()..],
                                ParseState::ParsingValue(n * decimal_shift + d),
                            )
                        }
                        None if n == 0 => (buf, ParseState::Seeking),
                        None => return Ok(n),
                    }
                }
            }
        }
    }
}

pub fn ram_readln<const N: usize>() -> io::Result<usize> {
    let mut file = File::open("/proc/meminfo")?;
    let mut buf = [0u8; N];
    let mut start = 0;

    loop {
        let read = match file.read(&mut buf[start..]) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(read) => read + start,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        start = 0;
        let line_iter = buf[..read].split_inclusive(|b| *b == b'\n');
        let mut start_ptr = 0;
        for line in line_iter {
            if line.last() != Some(&b'\n') {
                buf.copy_within(start_ptr..read, 0);
                start = read - start_ptr;
                break;
            }
            start_ptr += line.len();
            let Some(value) = line
                .strip_prefix(b"MemAvailable:")
                .map(|b| b.trim_ascii_start())
                .and_then(|b| b.split(|b| b.is_ascii_whitespace()).next())
                .and_then(|b| std::str::from_utf8(b).ok())
                .and_then(|p| p.parse().ok())
            else {
                continue;
            };
            return Ok(value);
        }
    }
}
