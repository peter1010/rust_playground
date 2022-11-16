use std::io;
use std::collections::HashMap;

use crate::conversion::{
    little_endian_2_bytes, 
    little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::{FileBlob, RawBlob};


pub struct MnemonicIndex {
    mnemonics : HashMap<u16, MnemonicIndexEntry>,
}


pub struct MnemonicIndexEntry {
    caption_off : u32,
    blob : RawBlob
}


pub struct MnemonicIndexIterator {
    items : Vec::<(u16, MnemonicIndexEntry)>
}


impl MnemonicIndex {

    pub fn from(fp : & mut FileBlob, schema : u16, root_font_family : u8) -> io::Result<MnemonicIndex>
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
        let mut mnemonics = HashMap::new();

        Self::validate_schema(schema, idx_entry_len, max_str_len); 

        for i in 0..num_entries {
            let (mnemonic, entry) = match schema {
                2 => {
                    MnemonicIndexEntry::load_v2(fp) ?
                }, 
                3 => {
                    MnemonicIndexEntry::load_v3(fp) ?
                }, 
                _ => panic!("Invalid schema")
            };
            let old = mnemonics.insert(mnemonic, entry);
            if old != None {
                panic!("Two entries with same mnemonic!");
            }
        }
        let mnemonic_index = MnemonicIndex { mnemonics };
        Result::Ok(mnemonic_index)
    }


    fn validate_schema(schema : u16, idx_entry_len : u8, max_str_len : u16) 
    {
        match schema {
            2 => if idx_entry_len != 6 { panic!("V2 MnemonicIndexEntry wrong size 4 != {}", idx_entry_len) },
            3 => if idx_entry_len != 5 { panic!("V3 MnemonicIndexEntry wrong size 3 != {}", idx_entry_len) },
            _ => panic!("Invalid format")
        };
        if max_str_len != 16 {
            panic!("Max string len should be 16 was {}", max_str_len);
        }
    }
}


impl IntoIterator for &MnemonicIndex {

    type Item = (u16, MnemonicIndexEntry);
    type IntoIter = MnemonicIndexIterator;

    fn into_iter(self) -> Self::IntoIter {
        let mut keys = Vec::new(); 
        for key in self.mnemonics.keys() {
            keys.push(*key)
        }
        keys.sort();
        keys.reverse();
        let mut items = Vec::new();
        for key in keys {
            items.push( (key, self.mnemonics[&key].clone()) );
        }
        MnemonicIndexIterator { items }
    }
}


impl MnemonicIndexEntry {

    pub fn get_caption_off(&self) -> u32
    {
        self.caption_off
    }

    pub fn to_string(&self) -> Result<String, String>
    {
        match self.blob.get_string(self.caption_off, 16) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Blob offset {} \n\t {}", self.caption_off, x))
        }
    }

    fn load_v2(fp : & mut FileBlob) -> io::Result<(u16, MnemonicIndexEntry)>
    {
        let mut buf = [0; 6];
        fp.read_exact(& mut buf) ?;
        let mnemonic = little_endian_2_bytes(&buf[0..2]);
        let offset = little_endian_4_bytes(&buf[2..6]);
        if offset == 0 { 
            panic!{"Empty slot"};
        };
        let entry = MnemonicIndexEntry { caption_off : offset, blob : fp.freeze() };
        Result::Ok((mnemonic, entry))
    }

    fn load_v3(fp : & mut FileBlob) -> io::Result<(u16, MnemonicIndexEntry)>
    {
        let mut buf = [0; 5];
        fp.read_exact(& mut buf) ?;
        let mnemonic = little_endian_2_bytes(&buf[0..2]);
        let offset = little_endian_3_bytes(&buf[2..5]);
        if offset == 0 { 
            panic!{"Empty slot"};
        };
        let entry = MnemonicIndexEntry { caption_off : offset, blob : fp.freeze()};
        Result::Ok((mnemonic, entry))
    } 
}


impl PartialEq for MnemonicIndexEntry {

    fn eq(&self, other : & Self) -> bool
    {
        self.caption_off == other.caption_off
    }
}


impl Clone for MnemonicIndexEntry {

    fn clone(&self) -> MnemonicIndexEntry 
    {
        MnemonicIndexEntry { caption_off : self.caption_off, blob : self.blob.clone() }
    }
}


impl Iterator for MnemonicIndexIterator {
    type Item = (u16, MnemonicIndexEntry);

    fn next(& mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}
