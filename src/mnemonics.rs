use std::collections::HashMap;

use crate::blob::{FileBlob, RawBlob, BlobRegions};

pub struct MnemonicIndex {
    values: HashMap<u8, MnemonicIndexEntry>,
}

pub struct MnemonicIndexEntry {
    caption_off: u32,
    tooltip_off: u32,
    blob: RawBlob,
}

pub struct MnemonicIndexIterator {
    values: Vec<(u8, MnemonicIndexEntry)>,
}

impl MnemonicIndex {
    ///
    /// Read and create a V4 MnemonicIndex.
    ///
    pub fn from(fp: &mut FileBlob) -> MnemonicIndex {
        let num_entries = fp.read_le_2bytes(BlobRegions::Mnemonics);
        let idx_entry_len = fp.read_byte(BlobRegions::Mnemonics);

//		println!("Number of entries {} size {}", num_entries, idx_entry_len);

        let mut values = HashMap::new();

        if idx_entry_len != 0 {
            Self::validate_schema(4, idx_entry_len);

            for _i in 0..num_entries {
                let (value, entry) = MnemonicIndexEntry::load(fp);
//				println!("{}", param);

                let old = values.insert(value, entry);
                if old != None {
                    panic!("Two entries with same mnemonic! item={}", value);
                }
            }

            MnemonicIndex { values }
        } else {
            MnemonicIndex { values }
        }
    }


    pub fn validate_schema(schema: u16, idx_entry_len: u8) {
        match schema {
            4 => {
                if idx_entry_len != 5 {
                    panic!("V4 MnemonicIndexEntry wrong size 3 != {}", idx_entry_len)
                }
            }
            _ => panic!("Invalid format"),
        };
    }

    pub fn get_num_values(&self) -> usize {
        self.values.len()
    }
}

impl Clone for MnemonicIndex{
    fn clone(&self) -> MnemonicIndex {
        let mut values = self.values.clone();

        //HashMap<u8, MnemonicIndexEntry>::new();

        //for entry = self.values {
        //    values.insert(
        //}
        MnemonicIndex {
            values
        }
    }
}


impl IntoIterator for &MnemonicIndex {
    type Item = (u8, MnemonicIndexEntry);
    type IntoIter = MnemonicIndexIterator;

    fn into_iter(self) -> Self::IntoIter {
        let mut keys = Vec::new();
        for key in self.values.keys() {
            keys.push(*key)
        }
        keys.sort();
        keys.reverse();
        let mut values = Vec::new();
        for key in keys {
            values.push((key, self.values[&key].clone()));
        }
        MnemonicIndexIterator { values }
    }
}

impl MnemonicIndexEntry {
    fn load(fp: &mut FileBlob) -> (u8, MnemonicIndexEntry) {
        let param = fp.read_byte(BlobRegions::Products);
        let caption_off = fp.read_le_3bytes(BlobRegions::Products);
        let tooltip_off = fp.read_le_3bytes(BlobRegions::Products);

//		println!("{} => {} {} {}", param, caption_off, tooltip_off, mnemonic_off);

        if caption_off == 0 {
            println!("Empty parameter?");
        };
        let param_entry = MnemonicIndexEntry {
            caption_off: caption_off,
            tooltip_off: tooltip_off,
            blob: fp.freeze(),
        };
        (param, param_entry)
    }


    pub fn to_string(&self) -> Result<String, String> {
        let str1 = match self.blob.get_string(self.caption_off, 32) {
            Ok(x) => x,
            Err(x) => return Err(format!("Blob offset {} \n\t {}", self.caption_off, x)),
        };
        if self.tooltip_off != 0 {
            let str2 = match self.blob.get_string(self.tooltip_off, 32) {
                Ok(x) => x,
                Err(x) => return Err(format!("Blob offset {} \n\t {}", self.tooltip_off, x)),
            };
            return Result::Ok(format!("{} / {}", str1, str2));
        };
        return Result::Ok(str1);
    }
}

impl PartialEq for MnemonicIndexEntry {
    fn eq(&self, other: &Self) -> bool {
        self.caption_off == other.caption_off
    }
}

impl Clone for MnemonicIndexEntry {
    fn clone(&self) -> MnemonicIndexEntry {
        MnemonicIndexEntry {
            caption_off: self.caption_off,
            tooltip_off: self.tooltip_off,
            blob: self.blob.clone(),
        }
    }
}

impl Iterator for MnemonicIndexIterator {
    type Item = (u8, MnemonicIndexEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.values.pop()
    }
}
