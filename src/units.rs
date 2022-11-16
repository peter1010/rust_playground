use std::io;
use std::collections::HashMap;

use crate::conversion::{
    little_endian_2_bytes, 
    little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::{FileBlob, RawBlob};



pub struct UnitsIndex {
    units : HashMap<u16, UnitsIndexEntry>,
}

pub struct UnitsIndexEntry {
    caption_off : u32,
    tooltip_off : u32,
    blob : RawBlob
}


pub struct UnitsIndexIterator {
    items : Vec::<(u16, UnitsIndexEntry)>
}


impl UnitsIndex {

    pub fn from(fp : & mut FileBlob, schema : u16, root_font_family : u8) -> io::Result<UnitsIndex>
    {
        let mut header = [0; 6];
        fp.read_exact(& mut header).expect("Failed to read Units header");

        let num_entries = little_endian_2_bytes(&header[0..2]);
        let max_str_len = little_endian_2_bytes(&header[2..4]);
        let font_family = header[4];
        let idx_entry_len = header[5];

        if root_font_family != font_family {
            panic!("Mis-match font_family");
        }
        let mut units = HashMap::new();
        
        Self::validate_schema(schema, idx_entry_len, max_str_len); 

        for i in 0..num_entries {
            let (unit_id, entry) = match schema {
                2 => {
                    UnitsIndexEntry::load_v2(fp) ?
                }, 
                3 => {
                    UnitsIndexEntry::load_v3(fp) ?
                }, 
                _ => panic!("Invalid schema")
            };
            let old = units.insert(unit_id, entry);
            if old != None {
                panic!("Two entries with same units!");
            }
        };
        Result::Ok(UnitsIndex { units })
    }
    

    fn validate_schema(schema : u16, idx_entry_len : u8, max_str_len : u16) 
    {
        match schema {
            2 => if idx_entry_len != 6 { panic!("V2 KeypadStrIndexEntry wrong size 4 != {}", idx_entry_len) },
            3 => if idx_entry_len != 5 { panic!("V3 KeypadStrIndexEntry wrong size 3 != {}", idx_entry_len) },
            _ => panic!("Invalid format")
        };

        if max_str_len != 16 {
            panic!("Units, max string len should be 16!");
        }
    }
}

impl IntoIterator for &UnitsIndex {

    type Item = (u16, UnitsIndexEntry);
    type IntoIter = UnitsIndexIterator;

    fn into_iter(self) -> Self::IntoIter {
        let mut keys = Vec::new(); 
        for key in self.units.keys() {
            keys.push(*key)
        }
        keys.sort();
        keys.reverse();
        let mut items = Vec::new();
        for key in keys {
            items.push( (key, self.units[&key].clone()) );
        }
        UnitsIndexIterator { items }
    }
}



impl UnitsIndexEntry {
    
    pub fn get_caption_off(&self) -> u32
    {
        self.caption_off
    }

    pub fn get_tooltip_off(&self) -> u32
    {
        self.caption_off
    }

    pub fn to_string(&self) -> Result<String, String>
    {
        let str1 = match self.blob.get_string(self.caption_off, 16) {
            Ok(x) => x,
            Err(x) => return Err(format!("Blob offset {} \n\t {}", self.caption_off, x))
        };
        if self.tooltip_off != 0 {
            let str2 = match self.blob.get_string(self.tooltip_off, 16) {
                Ok(x) => x,
                Err(x) => return Err(format!("Blob offset {} \n\t {}", self.tooltip_off, x))
            };
            return Result::Ok(format!("{} / {}", str1, str2));
        };
        return Result::Ok(str1);
    }



    fn load_v2(fp : & mut FileBlob) -> io::Result<(u16,UnitsIndexEntry)>
    {
        let mut buf = [0; 6];
        fp.read_exact(& mut buf) ?;
        let unit_id = little_endian_2_bytes(&buf[0..2]);
        let offset = little_endian_4_bytes(&buf[2..6]);
        if offset == 0 { 
            panic!{"Empty slot"};
        };
        let entry = UnitsIndexEntry { caption_off : offset, tooltip_off : 0, blob : fp.freeze()};
        Result::Ok((unit_id, entry))
    }

    fn load_v3(fp : & mut FileBlob) -> io::Result<(u16, UnitsIndexEntry)>
    {
        let mut buf = [0; 5];
        fp.read_exact(& mut buf) ?;
        let unit_id = little_endian_2_bytes(&buf[0..2]);
        let offset = little_endian_3_bytes(&buf[2..5]);
        if offset == 0 { 
            panic!{"Empty slot"};
        };
        let entry = UnitsIndexEntry { caption_off : offset, tooltip_off : 0, blob : fp.freeze()};
        Result::Ok((unit_id, entry))
    }
}

impl PartialEq for UnitsIndexEntry {

    fn eq(&self, other : & Self) -> bool
    {
        self.caption_off == other.caption_off
    }
}

impl Clone for UnitsIndexEntry {

    fn clone(&self) -> UnitsIndexEntry 
    {
        UnitsIndexEntry { caption_off : self.caption_off, tooltip_off : self.tooltip_off, blob : self.blob.clone() }
    }
}


impl Iterator for UnitsIndexIterator {
    type Item = (u16, UnitsIndexEntry);

    fn next(& mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}
