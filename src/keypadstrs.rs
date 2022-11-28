use std::collections::HashMap;

use crate::blob::{FileBlob, RawBlob, BlobRegions};

pub struct KeypadStrIndex 
{
    keypad_strs: HashMap<u16, KeypadStrIndexEntry>,
}

pub struct KeypadStrIndexEntry {
    caption_off: u32,
    blob: RawBlob,
}

pub struct KeypadStrIterator {
    items: Vec<(u16, KeypadStrIndexEntry)>,
}

impl KeypadStrIndex {
    pub fn from(fp: &mut FileBlob, schema: u16, root_font_family: u8) -> KeypadStrIndex {

        let num_entries = fp.read_le_2bytes(BlobRegions::KeypadStrs);
        let max_str_len = fp.read_le_2bytes(BlobRegions::KeypadStrs);
        let font_family = fp.read_byte(BlobRegions::KeypadStrs);
        let idx_entry_len = fp.read_byte(BlobRegions::KeypadStrs);

        if root_font_family != font_family {
            panic!("Mis-match font_family");
        }
        let mut keypad_strs = HashMap::new();

        Self::validate_schema(schema, idx_entry_len, max_str_len);

        for _i in 0..num_entries {
            let (string_id, entry) = match schema {
                2 => KeypadStrIndexEntry::load_v2(fp),
                _ => panic!("Invalid schema"),
            };
            let old = keypad_strs.insert(string_id, entry);
            if old != None {
                panic!("Two entries with same keypad strings!");
            }
        }
        KeypadStrIndex { keypad_strs }
    }

    fn validate_schema(schema: u16, idx_entry_len: u8, max_str_len: u16) {
        match schema {
            2 => {
                if idx_entry_len != 6 {
                    panic!("V2 KeypadStrIndexEntry wrong size 4 != {}", idx_entry_len)
                }
            }
            _ => panic!("Invalid format"),
        };
        if max_str_len != 32 {
            panic!("Keypad string len is incorrect");
        }
    }

    pub fn empty() -> KeypadStrIndex {
        let keypad_strs = HashMap::<u16, KeypadStrIndexEntry>::new();
        KeypadStrIndex { keypad_strs }
    }
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
            items.push((key, self.keypad_strs[&key].clone()));
        }
        KeypadStrIterator { items }
    }
}

impl KeypadStrIndexEntry {
    fn load_v2(fp: &mut FileBlob) -> (u16, KeypadStrIndexEntry) {
        let string_id = fp.read_le_2bytes(BlobRegions::KeypadStrs);
        let offset = fp.read_le_4bytes(BlobRegions::KeypadStrs);
        if offset == 0 {
            panic! {"Empty slot"};
        };
        let entry = KeypadStrIndexEntry {
            caption_off: offset,
            blob: fp.freeze(),
        };
        (string_id, entry)
    }

    pub fn to_string(&self) -> Result<String, String> {
        match self.blob.get_string(self.caption_off, 32) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Blob offset {} \n\t {}", self.caption_off, x)),
        }
    }
}

impl PartialEq for KeypadStrIndexEntry {
    fn eq(&self, other: &Self) -> bool {
        self.caption_off == other.caption_off
    }
}

impl Clone for KeypadStrIndexEntry {
    fn clone(&self) -> KeypadStrIndexEntry {
        KeypadStrIndexEntry {
            caption_off: self.caption_off,
            blob: self.blob.clone(),
        }
    }
}

impl Iterator for KeypadStrIterator {
    type Item = (u16, KeypadStrIndexEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}
