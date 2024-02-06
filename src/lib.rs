use std::{
    cmp, fmt,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    str,
};
use serde::{Deserialize, Serialize};

mod tests;

const BASE_N: u8 = 78;

const DNA_4BIT_DECODE_MAP: [u8; 16] = [0, 65, 67, 71, 84, 97, 99, 103, 116, 78, 110, 0, 0, 0, 0, 0];

pub const EMPTY_STRING: &str = "";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    None,
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMask {
    None,
    Lower,
    N,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub chr: String,
    pub start: i32,
    pub end: i32,
}

impl Location {
    pub fn new(chr: &str, start: i32, end: i32) -> Result<Location, String> {
        if !chr.contains("chr") {
            panic!("chr {} is invalid", chr);
        }

        let s: i32 = cmp::max(1, cmp::min(start, end));

        Ok(Location {
            chr:chr.to_string(),
            start: s,
            end: cmp::max(s, end),
        })
    }

    // pub fn chr(&self)->&str {
    //     return &self.chr;
    // }

    // pub fn start(&self)->u32 {
    //     return self.start;
    // }

    // pub fn end(&self)->u32 {
    //     return self.start;
    // }

    pub fn length(&self) -> i32 {
        return self.end - self.start + 1;
    }

    // Returns the mid point of the location.
    pub fn mid(&self) -> i32 {
        return (self.start + self.end) / 2;
    }

    // Converts a string location of the form "chr1:1-2" into a location struct.
    pub fn parse(location: &str) -> Result<Location, String> {
        if !location.contains(":") || !location.contains("chr") {
            panic!("invalid location format")
        }

        let tokens: Vec<&str> = location.split(":").collect();

        let chr: &str = tokens[0];

        let start: i32;
        let end: i32;

        if tokens[1].contains("-") {
            let range_tokens:Vec<&str> = tokens[1].split("-").collect();

            start = match range_tokens[0].parse() {
                Ok(start) => start,
                Err(_) => return Err("invalid location format".to_string()),
            };

            end = match range_tokens[1].parse() {
                Ok(start) => start,
                Err(_) => return Err("invalid location format".to_string()),
            };
        } else {
            start = match tokens[1].parse() {
                Ok(start) => start,
                Err(_) => return Err("invalid location format".to_string()),
            };

            end = start;
        }

        let loc = match Location::new(chr, start, end) {
            Ok(loc) => loc,
            Err(err) => return Err(format!("{}", err)),
        };

        Ok(loc)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}", self.chr, self.start, self.end)
    }
}

fn to_upper(b: u8) -> u8 {
    match b {
        97 => 65,
        99 => 67,
        103 => 71,
        116 => 84,
        110 => BASE_N,
        65 => 65,
        67 => 67,
        71 => 71,
        85 => 85,
        BASE_N => BASE_N,
        _ => 0,
    }
}

fn is_lower(b: u8) -> bool {
    match b {
        97 => true,
        99 => true,
        103 => true,
        116 => true,
        110 => true,
        _ => false,
    }
}

fn to_lower(b: u8) -> u8 {
    match b {
        65 => 97,
        67 => 99,
        71 => 103,
        84 => 116,
        BASE_N => 110,
        97 => 97,
        99 => 99,
        103 => 103,
        116 => 116,
        110 => 110,
        _ => 0,
    }
}

fn change_case(dna: &mut Vec<u8>, format: &Format, repeat_mask: &RepeatMask) {
    println!("{:?} {}", repeat_mask, *repeat_mask != RepeatMask::None);
    if *format == Format::None || *repeat_mask != RepeatMask::None {
        return;
    }

    for i in 0..dna.len() {
        match format {
            Format::Lower => dna[i] = to_lower(dna[i]),
            Format::Upper => dna[i] = to_upper(dna[i]),
            _ => (),
        }
    }
}

fn change_repeat_mask(dna: &mut Vec<u8>, repeat_mask: &RepeatMask) {
    if *repeat_mask == RepeatMask::N {
        for i in 0..dna.len() {
            if is_lower(dna[i]) {
                dna[i] = BASE_N
            }
        }
    }
}

fn comp_base(b: u8) -> u8 {
    match b {
        65 => 84,
        67 => 71,
        71 => 67,
        84 => 65,
        97 => 116,
        99 => 103,
        103 => 99,
        116 => 97,
        78 => 78,
        110 => 10,
        _ => 0,
    }
}

fn compliment(dna: &mut Vec<u8>) {
    for i in 0..dna.len() {
        dna[i] = comp_base(dna[i])
    }
}



pub struct DNA {
    dir: String,
}

impl DNA {
    pub fn new(dir: &str) -> DNA {
        return DNA { dir:dir.to_string() };
    }

    // fn rev_comp(&self, dna: &mut Vec<u8>) {
    //     dna.reverse();
    //     self.comp(dna);
    // }

    pub fn get_dna(
        &self,
        location: &Location,
        rev: bool,
        comp: bool,
        format: &Format,
        repeat_mask: &RepeatMask,
    ) -> Result<String, String> {
        let mut s: i32 = location.start - 1;
        let e: i32 = location.end - 1;
        let l: i32 = e - s + 1;
        let bs: i32 = s / 2;
        let be: i32 = e / 2;
        let bl: i32 = be - bs + 1;

        let mut d: Vec<u8> = vec![0; bl as usize];

        let file: String = match Path::new(&self.dir)
            .join(format!("{}.dna.4bit", location.chr.to_lowercase()))
            .to_str()
        {
            Some(s) => s.to_string(),
            None => return Err("cannot open file".to_string()),
        };

        let mut f: File = match File::open(file) {
            Ok(file) => file,
            Err(_) => return Err("cannot open file".to_string()),
        };

        match f.seek(SeekFrom::Start((1 + bs) as u64)) {
            Ok(_) => (),
            Err(_) => return Err("offset invalid".to_string()),
        };

        match f.read(&mut d) {
            Ok(_) => (),
            Err(_) => return Err("buffer invalid".to_string()),
        };

        let mut dna: Vec<u8> = vec![0; l as usize];

        // which byte we are scanning (each byte contains 2 bases)
        let mut byte_index: i32 = 0;
        let mut v: u8;
        let mut base_index: i32;

        for i in 0..l {
            // Which base we want in the byte
            // If the start position s is even, we want the first
            // 4 bits of the byte, else the lower 4 bits.
            base_index = s % 2;

            v = d[byte_index as usize];

            if base_index == 0 {
                v = v >> 4
            } else {
                // if we are on the second base of the byte, on the
                // next loop we must proceed to the next byte to get
                // the base
                byte_index += 1;
            }

            // mask for lower 4 bits since these
            // contain the dna base code
            dna[i as usize] = DNA_4BIT_DECODE_MAP[(v & 15) as usize];

            s += 1;
        }

        if rev {
            dna.reverse();
        }

        if comp {
            compliment(&mut dna)
        }

        change_repeat_mask(&mut dna, repeat_mask);

        change_case(&mut dna, format, repeat_mask);

        let s: String = match String::from_utf8(dna) {
            Ok(s) => s,
            Err(err) => return Err(err.to_string()),
        };

        Ok(s)
    }
}
