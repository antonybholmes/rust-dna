use std::{
    cmp,
    collections::HashMap,
    fmt,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    str,
};

mod tests;

const DNA_4BIT_DECODE_MAP: [u8; 16] = [0, 65, 67, 71, 84, 97, 99, 103, 116, 78, 110, 0, 0, 0, 0, 0];

pub const EMPTY_STRING: & str = "";

pub struct Location {
    pub chr: String,
    pub start: u32,
    pub end: u32,
}

impl Location {
    pub fn parse(location: &str) -> Location {
        let tokens: Vec<String> = location.split(":").map(String::from).collect();

        let chr: String = tokens[0].to_string();

        let tokens2: Vec<String> = tokens[1].split("-").map(String::from).collect();
        let start: u32 = tokens2[0].parse().unwrap_or(1);
        let end: u32 = tokens2[1].parse().unwrap_or(1);

        return Location {
            chr,
            start: cmp::min(start, end),
            end: cmp::max(start, end),
        };
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}", self.chr, self.start, self.end)
    }
}

pub struct DNA<'a> {
    dir: &'a str,
    compliment_map: HashMap<u8, u8>,
}

impl DNA<'_> {
    pub fn new(dir: &str) -> DNA {
        let m: HashMap<u8, u8> = HashMap::from([
            (0, 0),
            (65, 84),
            (67, 71),
            (71, 67),
            (84, 65),
            (97, 116),
            (99, 103),
            (103, 99),
            (116, 97),
            (78, 78),
            (110, 10),
        ]);

        return DNA {
            dir,
            compliment_map: m,
        };
    }

    fn comp(&self, dna: &mut Vec<u8>) {
        for i in 0..dna.len() as usize {
            dna[i] = self.compliment_map[&dna[i]]
        }
    }

    // fn rev_comp(&self, dna: &mut Vec<u8>) {
    //     dna.reverse();
    //     self.comp(dna);
    // }

    pub fn get_dna(&self, location: &Location, rev: bool, comp: bool) -> String {
        let mut s: u32 = location.start - 1;
        let e: u32 = location.end - 1;
        let l: u32 = e - s + 1;
        let bs: u32 = s / 2;
        let be: u32 = e / 2;
        let bl: u32 = be - bs + 1;

        let mut d: Vec<u8> = vec![0; bl as usize];

        let file: String = Path::new(&self.dir)
            .join(format!("{}.dna.4bit", location.chr.to_lowercase()))
            .to_str()
            .unwrap()
            .to_string();

        let mut f: File = match File::open(file) {
            Ok(file) => file,
            Err(_) => return EMPTY_STRING.to_string(),
        };

        match f.seek(SeekFrom::Start((1 + bs) as u64)) {
            Ok(_) => (),
            Err(_) => return EMPTY_STRING.to_string(),
        };

        match f.read(&mut d) {
            Ok(_) => (),
            Err(_) => return EMPTY_STRING.to_string(),
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
            self.comp(&mut dna)
        }

        return match str::from_utf8(&dna) {
            Ok(str) => str.to_string(),
            Err(_) => return EMPTY_STRING.to_string(),
        };
    }
}
