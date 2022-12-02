use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::characters::CharacterMaps;

#[derive(PartialEq, Clone, Copy, Debug)]
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
    KeypadStrs,
    Text,
    Invalid
}

///
/// Collect some stats
///
struct Stats {
    regions: Vec<BlobRegions>,
    string_offsets : HashMap<String, (u32, u32)>,
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

    fn read_exact(&mut self, buf: &mut [u8], region: BlobRegions)  {
        let to_read = buf.len();
        let pos = self.pos;

        for i in 0..to_read {
            buf[i] = self.data.data[pos + i];
        }
        self.pos = pos + to_read;

        self.data.add_region(pos, pos + to_read, region)
    }

    pub fn read_le_4bytes(&mut self, region: BlobRegions) -> u32 {
		let mut values = [0; 4];
   		self.read_exact(&mut values, region);
		return (values[0] as u32) | ((values[1] as u32) << 8) | ((values[2] as u32) << 16) | ((values[3] as u32) << 24);
	}
	
	pub fn read_le_3bytes(&mut self, region: BlobRegions) -> u32 {
		let mut values = [0; 3];
   		self.read_exact(&mut values, region);
		return (values[0] as u32) | ((values[1] as u32) << 8) | ((values[2] as u32) << 16);
	}
	
	pub fn read_le_2bytes(&mut self, region: BlobRegions) -> u16 {
		let mut values = [0; 2];
   		self.read_exact(&mut values, region);
		return (values[0] as u16) | ((values[1] as u16) << 8);
	}
	
	pub fn read_byte(&mut self, region: BlobRegions) -> u8 {
		let mut values = [0; 1];
   		self.read_exact(&mut values, region);
		return values[0];
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
        let size = data.len();
        if size != expected_size as usize {
            panic!("File length incorrect");
        }
        let stats = Stats { regions: vec![BlobRegions::Empty; size], string_offsets : HashMap::<String, (u32,u32)>::new()};
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

    ///
    /// Get bytes that represent a string, from the blob
    ///
    fn get_bytes(&self, off: u32, max_length: u16) -> Vec<u8> {
        let mut bytes = Vec::new();
        let buf = &self.data.data;

        let mut i = off as usize;
        let end = i + (max_length as usize);

        while i < end {
            let ch = buf[i];
            if ch == 0 {
                i += 1;
                break;
            } else {
                bytes.push(ch);
            }
            i += 1;
        }
        // Note down what was in that region of the Blob for diagnostics.
        self.data.add_region(off as usize, i, BlobRegions::Text);

        return bytes;
    }

    pub fn get_string(&self, off: u32, max_length: u16) -> Result<String, String> {
        if off == 0 {
            return Result::Ok("[-- no string --]".to_string());
        }
        let bytes = self.get_bytes(off, max_length);
        let len = bytes.len() as u32;
        if len == 0 {
            self.data.add_string("", off, 1);
            return Result::Ok("[-- empty string --]".to_string());
        }
        let result = self.bytes_to_string(bytes);
        match &result {
            Ok(x) => self.data.add_string(&x, off, len),
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

    pub fn add_string(&self, string: &str, off : u32, size : u32)
    {
        let mut stats = self.stats.borrow_mut();
        let string_off = &mut stats.string_offsets;
        match string_off.get(string) {
            Some(x) => {
                let (orig_off, count) = *x;
                if orig_off != off {
                    string_off.remove(string);
                    string_off.insert(string.to_string(), (orig_off, count + size));
                }
            },
            None => {string_off.insert(string.to_string(), (off, 0));}
        }
    }

    pub fn display_stats(&self)
    {
        let stats = self.stats.borrow_mut();
        let mut duplicate_count = 0;
        for x in &stats.string_offsets {
            let (string, (orig_off, count)) = x;
            if *count > 1 {
                duplicate_count += count - 1;
                println!("{} duplicated {} times", string, count);
            }
        }
      
        println!("Duplicate count {}", duplicate_count);

        let mut unused = 0;
        let mut current_region = BlobRegions::Invalid;
        let mut pos = 0;
        let mut region_start = 0;
		let mut prelude = String::new();

        for x in &stats.regions {
            
            let reg = *x;

            if reg == BlobRegions::Empty {
                unused += 1;
            }
            if reg != current_region {
                if pos > region_start {
                    let text = format!("Region from {} to {} is {:?}", region_start, pos-1, current_region);
					if current_region == BlobRegions::Empty {
						println!("{}", prelude);
						println!("{}", text);
					} else {
						prelude = text;
					}
                    current_region = reg;
                    region_start = pos;
                }
            }
            pos += 1;
        }

        if unused > 0 {
            println!("{} bytes unused, {} wasted duplication", unused, duplicate_count);
        }
    }
}
