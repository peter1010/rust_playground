use std::io;
use std::collections::HashMap;

use crate::conversion::{
    little_endian_2_bytes, 
    little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::{FileBlob, RawBlob};



pub struct KeypadStrIndex {
    keypad_strs : HashMap<u16, KeypadStrIndexEntry>,
    max_str_len : u16,
    char_map : u8
}

struct KeypadStrIndexEntry {
    caption_off : u32,
    blob : RawBlob
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
        
        println!("- - max str len {}", max_str_len);
        Self::validate_schema(schema, idx_entry_len); 

        for i in 0..num_entries {
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
        Result::Ok(KeypadStrIndex { keypad_strs, max_str_len, char_map : font_family })
    }
    
    fn validate_schema(schema : u16, idx_entry_len : u8) 
    {
        match schema {
            2 => if idx_entry_len != 6 { panic!("V2 KeypadStrIndexEntry wrong size 4 != {}", idx_entry_len) },
            3 => if idx_entry_len != 5 { panic!("V3 KeypadStrIndexEntry wrong size 3 != {}", idx_entry_len) },
            _ => panic!("Invalid format")
        };
    }

    pub fn empty() -> KeypadStrIndex
    {
        let keypad_strs = HashMap::<u16, KeypadStrIndexEntry>::new();
        KeypadStrIndex { keypad_strs, max_str_len : 0, char_map : 0}
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


impl KeypadStrIndexEntry {

    fn load_v2(fp : & mut FileBlob) -> io::Result<(u16, KeypadStrIndexEntry)>
    {
        let mut buf = [0; 6];
        fp.read_exact(& mut buf) ?;
        let string_id = little_endian_2_bytes(&buf[0..2]);
        println!("string_id {}", string_id);
        let offset = little_endian_4_bytes(&buf[2..6]);
        if offset == 0 { 
            println!{"Empty slot"};
        };
        let entry = KeypadStrIndexEntry { caption_off : offset, blob : fp.freeze()};
        return Result::Ok((string_id, entry))
    }

    fn load_v3(fp : & mut FileBlob) -> io::Result<(u16, KeypadStrIndexEntry)>
    {
        let mut buf = [0; 5];
        fp.read_exact(& mut buf) ?;
        let string_id = little_endian_2_bytes(&buf[0..2]);
        println!("string_id {}", string_id);
        let offset = little_endian_3_bytes(&buf[2..5]);
        if offset == 0 {
            println!{"Empty slot"};
        };
        let entry = KeypadStrIndexEntry { caption_off : offset, blob : fp.freeze()};
        return Result::Ok((string_id, entry))
    }
} 

impl PartialEq for KeypadStrIndexEntry {

    fn eq(&self, other : & Self) -> bool
    {
        self.caption_off == other.caption_off
    }
}
