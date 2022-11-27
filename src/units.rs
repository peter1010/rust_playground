use std::collections::{HashMap, HashSet};

use crate::conversion::{little_endian_2_bytes, little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::{FileBlob, RawBlob, BlobRegions};

pub struct UnitsIndex 
{
    units: HashMap<u16, UnitsIndexEntry>,
}

pub struct UnitsIndexEntry 
{
    units: u16,
    caption_off: u32,
    tooltip_off: u32,
    blob: RawBlob,
}

pub struct UnitsIndexIterator 
{
    items: Vec<(u16, UnitsIndexEntry)>,
}

impl UnitsIndex {

    pub fn new(units : HashMap<u16, UnitsIndexEntry>) -> UnitsIndex
    {
        let mut hits = HashSet::<u16>::new();

        for entry in &units {
            let units = entry.1.units;

            assert_eq!(*entry.0, units);

            if hits.contains(&units) {
                panic!("Duplicate units detected");
            }
            hits.insert(units);
        }
        UnitsIndex { units }
    }


    pub fn from(fp: &mut FileBlob, schema: u16, root_font_family: u8) -> UnitsIndex {
		
		let num_entries = fp.read_le_2bytes(BlobRegions::Units);
		println!("Num entries {}", num_entries);
        
		let mut max_str_len = 256;
		if schema < 4 {
        	max_str_len = fp.read_le_2bytes(BlobRegions::Units);
        	let font_family = fp.read_byte(BlobRegions::Units);
        
			if root_font_family != font_family {
            	panic!("Mis-match font_family");
        	}
		}

        let idx_entry_len = fp.read_byte(BlobRegions::Units);
        
		Self::validate_schema(schema, idx_entry_len, max_str_len);

        let mut units = HashMap::new();

        for _i in 0..num_entries {
            let (unit_id, entry) = match schema {
                2 => UnitsIndexEntry::load_v2(fp),
                3 => UnitsIndexEntry::load_v3(fp),
				4 => UnitsIndexEntry::load_v4(fp),
                _ => panic!("Invalid schema"),
            };
            units.insert(unit_id, entry);
        }
        UnitsIndex::new(units)
    }

    fn validate_schema(schema: u16, idx_entry_len: u8, max_str_len: u16) {
		let mut req_str_len = 16;
        match schema {
            2 => {
                if idx_entry_len != 6 {
                    panic!("V2 UnitsIndexEntry wrong size 6 != {}", idx_entry_len)
                }
            }
            3 => {
                if idx_entry_len != 5 {
                    panic!("V3 UnitsIndexEntry wrong size 5 != {}", idx_entry_len)
                }
            }
            4 => {
                if idx_entry_len != 8 {
                    panic!("V4 UnitsIndexEntry wrong size 8 != {}", idx_entry_len)
                }
				req_str_len = 256;
            }
            _ => panic!("Invalid format, schema = {}", schema),
        };

        if max_str_len != req_str_len {
            panic!("Units, max string len should be {} not {}!", req_str_len, max_str_len);
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
            items.push((key, self.units[&key].clone()));
        }
        UnitsIndexIterator { items }
    }
}

impl UnitsIndexEntry {

    pub fn new(units: u16, caption_off: u32, tooltip_off: u32, fp : & mut FileBlob) -> UnitsIndexEntry
    {
        UnitsIndexEntry {
            units,
            caption_off,
            tooltip_off,
            blob: fp.freeze()
        }
    }

    pub fn get_caption_off(&self) -> u32 {
        self.caption_off
    }

    pub fn get_tooltip_off(&self) -> u32 {
        self.caption_off
    }

    pub fn to_string(&self) -> Result<String, String> {
        let str1 = match self.blob.get_string(self.caption_off, 16) {
            Ok(x) => x,
            Err(x) => return Err(format!("Blob offset {} \n\t {}", self.caption_off, x)),
        };
        if self.tooltip_off != 0 {
            let str2 = match self.blob.get_string(self.tooltip_off, 16) {
                Ok(x) => x,
                Err(x) => return Err(format!("Blob offset {} \n\t {}", self.tooltip_off, x)),
            };
            return Result::Ok(format!("{} / {}", str1, str2));
        };
        return Result::Ok(str1);
    }

    fn load_v2(fp: &mut FileBlob) -> (u16, UnitsIndexEntry) {
        let mut buf = [0; 6];
        fp.read_exact(&mut buf, BlobRegions::Units);
        let unit_id = little_endian_2_bytes(&buf[0..2]);
        let offset = little_endian_4_bytes(&buf[2..6]);
        if offset == 0 {
            panic! {"Empty slot"};
        };
        let entry = UnitsIndexEntry::new(unit_id, offset, 0, fp);
        (unit_id, entry)
    }

    fn load_v3(fp: &mut FileBlob) -> (u16, UnitsIndexEntry) {
        let mut buf = [0; 5];
        fp.read_exact(&mut buf, BlobRegions::Units);
        let unit_id = little_endian_2_bytes(&buf[0..2]);
        let offset = little_endian_3_bytes(&buf[2..5]);
        if offset == 0 {
            panic! {"Empty slot"};
        };
        let entry = UnitsIndexEntry::new(unit_id, offset, 0, fp);
        (unit_id, entry)
    }
    fn load_v4(fp: &mut FileBlob) -> (u16, UnitsIndexEntry) {
        let unit_id = fp.read_le_2bytes(BlobRegions::Units);
        let caption_off = fp.read_le_3bytes(BlobRegions::Units);
        let tooltip_off = fp.read_le_3bytes(BlobRegions::Units);
        if caption_off == 0 {
            panic! {"Empty slot"};
        };
        let entry = UnitsIndexEntry::new(unit_id, caption_off, tooltip_off, fp);
        (unit_id, entry)
    }
}

impl PartialEq for UnitsIndexEntry {
    fn eq(&self, other: &Self) -> bool {
        self.caption_off == other.caption_off
    }
}

impl Clone for UnitsIndexEntry {
    fn clone(&self) -> UnitsIndexEntry {
        UnitsIndexEntry {
            units: self.units,
            caption_off: self.caption_off,
            tooltip_off: self.tooltip_off,
            blob: self.blob.clone(),
        }
    }
}

impl Iterator for UnitsIndexIterator {
    type Item = (u16, UnitsIndexEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}
