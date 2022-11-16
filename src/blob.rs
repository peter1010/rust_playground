use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::io;
use std::rc::Rc;

use crate::characters::CharacterMaps;


struct _Blob {
    data : Vec::<u8>,
    maps : CharacterMaps
}

pub struct FileBlob {
    data : Rc<_Blob>,
    pos : usize
}

pub struct RawBlob {
    data : Rc<_Blob>
}


impl FileBlob {

    pub fn set_pos(& mut self, pos : u32)
    {
        self.pos = pos as usize;
    }

    pub fn freeze(& mut self) -> RawBlob
    {
        RawBlob { data : self.data.clone() }
    }

    pub fn read_exact(& mut self, buf : & mut [u8]) -> io::Result<usize> 
    {
        let to_read = buf.len();
        let pos = self.pos;

        for i in 0..to_read {
            buf[i] = self.data.data[pos + i];
        }
        self.pos = pos + to_read;

        Result::Ok(buf.len())
    }


    ///
    /// Reads the whole file into Blob
    ///
    pub fn load(fp : & mut File, expected_size : u32, expected_crc : u32, maps : CharacterMaps) -> io::Result<FileBlob>
    {
        fp.seek(SeekFrom::Start(0)) ?;
        let mut buf = [0; 2048];
        let mut data = Vec::<u8>::new();
        loop {
            match fp.read(& mut buf) {
                Ok(len) => {
                    if len > 0 {
                        data.extend(& buf[..len]);
                    } else {
                        break;
                    }
                },
                Err(_) => {
                    break;
                }
            };
        }
        if data.len() != expected_size as usize {
            panic!("File length incorrect");
        }
        let _blob = Rc::new(_Blob {data, maps});

        Result::Ok( FileBlob { data : _blob, pos : 0})
    }
}

impl Clone for RawBlob {

    fn clone(&self) -> RawBlob
    {
        RawBlob { data : self.data.clone() }
    }
}

impl RawBlob {

    fn get_bytes(&self, off: u32, max_length : u16) -> Vec::<u8>
    {
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


    pub fn get_string(&self, off: u32, max_length : u16) -> Result<String,String>
    {
        if off == 0 {
            return Result::Ok("[-- no string --]".to_string());
        }
        let bytes = self.get_bytes(off, max_length);

        if bytes.len() == 0 {
            return Result::Ok("[-- empty string --]".to_string());
        }

        if self.data.maps.is_utf8() {
            return match String::from_utf8(bytes) {
                Ok(x) => Ok(x),
                Err(_) => Err("Failed to decode UTF-8 string".to_string())
            };
        } 
        
        let mut result = String::new();
        let mut i = 0;
        while i < bytes.len() {
            let ch = bytes[i];
            i += 1;
            let unicode = if (ch & 0xC0) == 0xC0 {
                if i == bytes.len() {
                    return Err(format!("Dangling half word character, string so far is {} from {:02X?}", result, bytes));
                }
                let mut ch2 = bytes[i] as u16;
                i += 1;
                ch2 = (((ch as u16) & !0xC0) << 7) | (ch2 >> 1);
                self.data.maps.decode_2bytes(ch2)
            } else {
                self.data.maps.decode_byte(ch)
            };
            result = match unicode { 
                Some(ch) => result + &ch, 
                None => result
            };
        }
        return Ok(result);
    }
}
