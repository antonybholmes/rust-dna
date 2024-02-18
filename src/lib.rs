use std::{
    cmp, error::Error, fmt::{self, Display}, fs::File, io::{Read, Seek, SeekFrom}, path::Path, str::{self, FromStr}
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
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone)]
pub enum DnaError {
    DatabaseError(String),
    LocationError(String),
    FormatError(String),
}

impl Error for DnaError {}

impl Display for DnaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnaError::DatabaseError(error) => write!(f, "{}", error),
            DnaError::LocationError(error) => write!(f, "{}", error),
            DnaError::FormatError(error) => write!(f, "{}", error),
        }
    }
}

pub type DnaResult<T> = Result<T, DnaError>;

impl Location {
    pub fn new(chr: &str, start: u32, end: u32) -> DnaResult<Self> {
        if !chr.contains("chr") {
            return Err(DnaError::LocationError(format!("chr {} is invalid", chr)));
        }

        let s: u32 = cmp::max(1, cmp::min(start, end));

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

    pub fn length(&self) -> u32 {
        return self.end - self.start + 1;
    }

    // Returns the mid point of the location.
    pub fn mid(&self) -> u32 {
        return (self.start + self.end) / 2;
    }

    // Converts a string location of the form "chr1:1-2" into a location struct.
    pub fn parse(location: &str) -> DnaResult<Location> {
        if !location.contains(":") || !location.contains("chr") {
            return Err(DnaError::LocationError(format!("invalid location format")));
        }

        let tokens: Vec<&str> = location.split(":").collect();

        let chr: &str = tokens[0];

        let start: u32;
        let end: u32;

        if tokens[1].contains("-") {
            let range_tokens:Vec<&str> = tokens[1].split("-").collect();

            start = unwrap_location(range_tokens[0])?;

            end = unwrap_location(range_tokens[1])?;
        } else {
            start = unwrap_location(tokens[1])?;

            end = start;
        }

        let loc: Location = Location::new(chr, start, end)?;

        Ok(loc)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}", self.chr, self.start, self.end)
    }
}

fn unwrap_location<T:FromStr>(token:&str) -> DnaResult<T> {
     match token.parse() {
        Ok(start) => Ok(start),
        Err(_) => Err(DnaError::LocationError("invalid location format".to_string())),
    }
}

#[derive(Serialize)]
pub struct DnaSeq {
    pub location: Location,
    pub dna: String,
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

fn change_repeat_mask(dna: &mut Vec<u8>, repeat_mask: &RepeatMask) {
    if *repeat_mask == RepeatMask::N {
        for i in 0..dna.len() {
            if is_lower(dna[i]) {
                dna[i] = BASE_N
            }
        }
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



pub struct DnaDb {
    dir: String,
}

impl DnaDb {
    pub fn new(dir: &str) -> Self {
        return DnaDb { dir:dir.to_string() };
    }

    // fn rev_comp(&self, dna: &mut Vec<u8>) {
    //     dna.reverse();
    //     self.comp(dna);
    // }

    pub fn dna(
        &self,
        location: &Location,
        rev: bool,
        comp: bool,
        format: &Format,
        repeat_mask: &RepeatMask,
    ) -> DnaResult<String> {
        let mut s: u32 = location.start - 1;
        let e: u32 = location.end - 1;
        let l: u32 = e - s + 1;
        let bs: u32 = s / 2;
        let be: u32 = e / 2;
        let bl: u32 = be - bs + 1;

        let mut d: Vec<u8> = vec![0; bl as usize];

        let file: String = match Path::new(&self.dir)
            .join(format!("{}.dna.4bit", location.chr.to_lowercase()))
            .to_str()
        {
            Some(s) => s.to_string(),
            None => return Err(DnaError::DatabaseError("cannot open file".to_string())),
        };

        let mut f: File = match File::open(file) {
            Ok(file) => file,
            Err(_) => return Err(DnaError::DatabaseError("cannot open file".to_string())),
        };

        match f.seek(SeekFrom::Start((1 + bs) as u64)) {
            Ok(_) => (),
            Err(_) => return Err(DnaError::DatabaseError("offset invalid".to_string())),
        };

        match f.read(&mut d) {
            Ok(_) => (),
            Err(_) => return Err(DnaError::DatabaseError("buffer invalid".to_string())),
        };

        let mut dna: Vec<u8> = vec![0; l as usize];

        // which byte we are scanning (each byte contains 2 bases)
        let mut byte_index: u32 = 0;
        let mut v: u8;
        let mut base_index: u32;

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
            Err(err) => return Err(DnaError::FormatError(err.to_string())),
        };

        Ok(s)
    }
}
