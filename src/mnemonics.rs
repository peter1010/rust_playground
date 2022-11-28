use std::collections::HashMap;

use crate::blob::{FileBlob, RawBlob, BlobRegions};

pub struct MnemonicIndex 
{
    values: HashMap<i32, MnemonicIndexEntry>,
}

pub struct MnemonicIndexEntry 
{
    value : i32,
    caption_off: u32,
    tooltip_off: u32,
    blob: RawBlob,
}

pub struct MnemonicIndexIterator 
{
    values: Vec<(i32, MnemonicIndexEntry)>,
}

impl MnemonicIndex 
{
    pub fn empty() -> MnemonicIndex
    {
        let mut values = HashMap::<i32, MnemonicIndexEntry>::new();
        MnemonicIndex
        {
            values
        }
    }

    pub fn new(values: HashMap::<i32, MnemonicIndexEntry>) -> MnemonicIndex
    {
        for entry in &values {
            let value = entry.1.value;

            assert_eq!(*entry.0, value);
        }
        MnemonicIndex { values }
    }


    ///
    /// Read and create a V4 MnemonicIndex.
    ///
    pub fn from(fp: &mut FileBlob) -> MnemonicIndex 
    {
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

            MnemonicIndex::new(values)
        } else {
            MnemonicIndex::new(values)
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

impl Clone for MnemonicIndex
{
    fn clone(&self) -> MnemonicIndex {
        let values = self.values.clone();

        MnemonicIndex {
            values
        }
    }
}


impl IntoIterator for &MnemonicIndex 
{
    type Item = (i32, MnemonicIndexEntry);
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

impl MnemonicIndexEntry 
{
    fn load(fp: &mut FileBlob) -> (i32, MnemonicIndexEntry) 
    {
        let value = fp.read_le_4bytes(BlobRegions::Mnemonics);
        let caption_off = fp.read_le_3bytes(BlobRegions::Mnemonics);
        let tooltip_off = fp.read_le_3bytes(BlobRegions::Mnemonics);

        let value : i32 = if value > 0x7FFFFFF {
            -((0xFFFFFFFF - value) as i32)
        } else {
            value as i32
        };

//		println!("{} => {} {} {}", param, caption_off, tooltip_off, mnemonic_off);

        if caption_off == 0 {
            println!("Empty parameter?");
        };
        let entry = MnemonicIndexEntry {
            value,
            caption_off: caption_off,
            tooltip_off: tooltip_off,
            blob: fp.freeze(),
        };
        (value, entry)
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
            value : self.value,
            caption_off: self.caption_off,
            tooltip_off: self.tooltip_off,
            blob: self.blob.clone(),
        }
    }
}

impl Iterator for MnemonicIndexIterator 
{
    type Item = (i32, MnemonicIndexEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.values.pop()
    }
}
