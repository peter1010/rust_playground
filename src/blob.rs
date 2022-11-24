use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::characters::CharacterMaps;

#[derive(PartialEq, Clone, Copy)]
pub enum BlobRegions {
    Empty,
    Header,
    Units,
    Products,
    Parameters,
    Menus,
    Modes,
    Enumerations,
    Mnemonics,
    KeypadStrs
}

struct Stats {
    regions: Vec<BlobRegions>,
    string_offsets : HashMap<String, u32>,
    duplicate_count : u32
}

struct _Blob {
    data: Vec<u8>,
    maps: CharacterMaps,
    stats: RefCell<Stats>
}

pub struct FileBlob {
    data: Rc<_Blob>,
    pos: usize,
}

pub struct RawBlob {
    data: Rc<_Blob>,
}

impl FileBlob {
    pub fn set_pos(&mut self, pos: u32) {
        self.pos = pos as usize;
    }

    pub fn freeze(&mut self) -> RawBlob {
        RawBlob {
            data: self.data.clone(),
        }
    }

    pub fn read_exact(&mut self, buf: &mut [u8], region: BlobRegions)  {
        let to_read = buf.len();
        let pos = self.pos;

        for i in 0..to_read {
            buf[i] = self.data.data[pos + i];
        }
        self.pos = pos + to_read;

        self.data.add_region(pos, pos + to_read, region)
    }

    ///
    /// Reads the whole file into Blob
    ///
    pub fn load(
        fp: &mut File,
        expected_size: u32,
        expected_crc: u32,
        maps: CharacterMaps,
    ) -> io::Result<FileBlob> {
        fp.seek(SeekFrom::Start(0))?;
        let mut buf = [0; 2048];
        let mut data = Vec::<u8>::new();
        loop {
            match fp.read(&mut buf) {
                Ok(len) => {
                    if len > 0 {
                        data.extend(&buf[..len]);
                    } else {
                        break;
                    }
                }
                Err(_) => {
                    break;
                }
            };
        }
        if data.len() != expected_size as usize {
            panic!("File length incorrect");
        }
        let size = data.len();
        let stats = Stats { regions: vec![BlobRegions::Empty; size], string_offsets : HashMap::<String, u32>::new(), duplicate_count : 0};
        let _blob = Rc::new(_Blob { data, maps, stats : RefCell::new(stats) });

        Result::Ok(FileBlob {
            data: _blob,
            pos: 0,
        })
    }

    pub fn display_stats(&self)
    {
        self.data.display_stats();
    }
}

impl Clone for RawBlob {
    fn clone(&self) -> RawBlob {
        RawBlob {
            data: self.data.clone(),
        }
    }
}

impl RawBlob {
    fn get_bytes(&self, off: u32, max_length: u16) -> Vec<u8> {
        let mut bytes = Vec::new();
        let buf = &self.data.data;

        let mut i = off as usize;
        let end = i + (max_length as usize);

        while i < end {
            let ch = buf[i];
            if ch == 0 {
                break;
            } else {
                bytes.push(ch);
            }
            i += 1;
        }
        return bytes;
    }

    pub fn get_string(&self, off: u32, max_length: u16) -> Result<String, String> {
        if off == 0 {
            return Result::Ok("[-- no string --]".to_string());
        }
        let bytes = self.get_bytes(off, max_length);

        if bytes.len() == 0 {
            self.data.add_string("", off);
            return Result::Ok("[-- empty string --]".to_string());
        }
        let result = self.bytes_to_string(bytes);
        match &result {
            Ok(x) => self.data.add_string(&x, off),
            Err(_) => {}  
        }
        return result;
    }


    fn bytes_to_string(&self, bytes : Vec<u8>) -> Result<String, String> {
        if self.data.maps.is_utf8() {
            return match String::from_utf8(bytes) {
                Ok(x) => Ok(x),
                Err(_) => Err("Failed to decode UTF-8 string".to_string()),
            };
        }

        let mut result = String::new();
        let mut i = 0;

        while i < bytes.len() {
            let ch1 = bytes[i];
            i += 1;
            let unicode = if i < bytes.len() {
                let ch2 = bytes[i];
                if ((ch2 & 0xC0) == 0xC0) && ((ch1 & 0x01) == 0x01) {
                    i += 1;
                    self.data
                        .maps
                        .decode_2bytes((((ch2 as u16) & !0xC0) << 7) | ((ch1 >> 1) as u16))
                } else if (ch1 & 0xC0) == 0xC0 {
                    return Err(format!(
                        "Dangling half word character, string so far is {} from {:02X?}",
                        result, bytes
                    ));
                } else {
                    self.data.maps.decode_byte(ch1)
                }
            } else if (ch1 & 0xC0) == 0xC0 {
                return Err(format!(
                    "Dangling half word character, string so far is {} from {:02X?}",
                    result, bytes
                ));
            } else {
                self.data.maps.decode_byte(ch1)
            };
            result = match unicode {
                Some(ch) => result + &ch,
                None => result,
            };
        }
        return Ok(result);
    }
}

impl _Blob {
    pub fn add_region(&self, start: usize, end: usize, _type: BlobRegions)
    {
        let regions = &mut self.stats.borrow_mut().regions;

        for i in start..end {
            if regions[i] == BlobRegions::Empty {
                regions[i] = _type;
            } else {
                if regions[i] != _type {
                    panic!("Region type mismatch")
                }
            }
        }
    }

    pub fn add_string(&self, string: &str, off : u32)
    {
        let mut stats = self.stats.borrow_mut();
        let string_off = &mut stats.string_offsets;
        match string_off.get(string) {
            Some(x) => if *x != off {
                stats.duplicate_count += 1;
                println!("!!----- Same string different offset ------!!")
            },
            None => {string_off.insert(string.to_string(), off);}
        }
    }

    pub fn display_stats(&self)
    {
        let stats = self.stats.borrow_mut();
        println!("Duplicate count {}", stats.duplicate_count);
    }
}
