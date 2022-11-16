use std::io;
use std::collections::HashMap;

use crate::conversion::{
    little_endian_2_bytes, 
    little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::{FileBlob, RawBlob};



pub struct KeypadStrIndex {
    keypad_strs : HashMap<u16, KeypadStrIndexEntry>
}

pub struct KeypadStrIndexEntry {
    caption_off : u32,
    blob : RawBlob
}

pub struct KeypadStrIterator {
    items : Vec<(u16, KeypadStrIndexEntry)>
}

impl KeypadStrIndex {

    pub fn from(fp : & mut FileBlob, schema : u16, root_font_family : u8) -> io::Result<KeypadStrIndex>
    {
        let mut header = [0; 6];
        fp.read_exact(& mut header) ?;

        let num_entries = little_endian_2_bytes(&header[0..2]);
        let max_str_len = little_endian_2_bytes(&header[2..4]);
        let font_family = header[4];
        let idx_entry_len = header[5];

        if root_font_family != font_family {
            panic!("Mis-match font_family");
        }
        let mut keypad_strs = HashMap::new();
        
        Self::validate_schema(schema, idx_entry_len, max_str_len); 

        for _i in 0..num_entries {
            let (string_id, entry) = match schema {
                2 => {
                    KeypadStrIndexEntry::load_v2(fp) ?
                }, 
                3 => {
                    KeypadStrIndexEntry::load_v3(fp) ?
                }, 
                _ => panic!("Invalid schema")
            };
            let old = keypad_strs.insert(string_id, entry);
            if old != None {
                panic!("Two entries with same keypad strings!");
            }
        }
        Result::Ok(KeypadStrIndex { keypad_strs })
    }
    
    fn validate_schema(schema : u16, idx_entry_len : u8, max_str_len : u16) 
    {
        match schema {
            2 => if idx_entry_len != 6 { panic!("V2 KeypadStrIndexEntry wrong size 4 != {}", idx_entry_len) },
            3 => if idx_entry_len != 5 { panic!("V3 KeypadStrIndexEntry wrong size 3 != {}", idx_entry_len) },
            _ => panic!("Invalid format")
        };
        if max_str_len != 32 {
            panic!("Keypad string len is incorrect");
        }
    }

    pub fn empty() -> KeypadStrIndex
    {
        let keypad_strs = HashMap::<u16, KeypadStrIndexEntry>::new();
        KeypadStrIndex { keypad_strs }
    }

//    pub fn get_caption(&self, string_id : u16) -> Option<String>
//    {
//        let entry = match self.keypad_strs.get(&string_id) {
//            Some(entry) => entry,
//            None => return None
//        };
//        let off = entry.caption_off;
//        let caption = entry.blob.get_raw_string(off, self.max_str_len);
//    }
}

impl IntoIterator for &KeypadStrIndex {

    type Item = (u16, KeypadStrIndexEntry);
    type IntoIter = KeypadStrIterator;

    fn into_iter(self) -> Self::IntoIter {
        let mut keys = Vec::new(); 
        for key in self.keypad_strs.keys() {
            keys.push(*key)
        }
        keys.sort();
        keys.reverse();
        let mut items = Vec::new();
        for key in keys {
            items.push( (key, self.keypad_strs[&key].clone()) );
        }
        KeypadStrIterator { items }
    }
}


impl KeypadStrIndexEntry {

    fn load_v2(fp : & mut FileBlob) -> io::Result<(u16, KeypadStrIndexEntry)>
    {
        let mut buf = [0; 6];
        fp.read_exact(& mut buf) ?;
        let string_id = little_endian_2_bytes(&buf[0..2]);
        let offset = little_endian_4_bytes(&buf[2..6]);
        if offset == 0 { 
            panic!{"Empty slot"};
        };
        let entry = KeypadStrIndexEntry { caption_off : offset, blob : fp.freeze()};
        return Result::Ok((string_id, entry))
    }

    fn load_v3(fp : & mut FileBlob) -> io::Result<(u16, KeypadStrIndexEntry)>
    {
        let mut buf = [0; 5];
        fp.read_exact(& mut buf) ?;
        let string_id = little_endian_2_bytes(&buf[0..2]);
        let offset = little_endian_3_bytes(&buf[2..5]);
        if offset == 0 {
            panic!{"Empty slot"};
        };
        let entry = KeypadStrIndexEntry { caption_off : offset, blob : fp.freeze()};
        return Result::Ok((string_id, entry))
    }
    
    pub fn to_string(&self) -> Result<String, String>
    {
        match self.blob.get_string(self.caption_off, 32) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Blob offset {} \n\t {}", self.caption_off, x))
        }
    }
} 


impl PartialEq for KeypadStrIndexEntry {

    fn eq(&self, other : & Self) -> bool
    {
        self.caption_off == other.caption_off
    }
}

impl Clone for KeypadStrIndexEntry {

    fn clone(&self) -> KeypadStrIndexEntry 
    {
        KeypadStrIndexEntry { caption_off : self.caption_off, blob : self.blob.clone() }
    }
}


impl Iterator for KeypadStrIterator {
    type Item = (u16, KeypadStrIndexEntry);

    fn next(& mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}
