use std::collections::HashMap;

use crate::conversion::{little_endian_2_bytes, little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::{FileBlob, RawBlob, BlobRegions};

///
/// This is a table of string ID to string lookups, primary
/// to handle string ID values from the drive.
///
pub struct EnumerationsIndex {
    enumerations: HashMap<u16, EnumerationsIndexEntry>,
}

pub struct EnumerationsIndexEntry {
    caption_off: u32,
    blob: RawBlob,
}

pub struct EnumerationsIndexIterator {
    items: Vec<(u16, EnumerationsIndexEntry)>,
}

impl EnumerationsIndex {
    pub fn from(fp: &mut FileBlob, schema: u16, root_font_family: u8) -> EnumerationsIndex {
        let mut common_hdr = [0; 2];
        fp.read_exact(&mut common_hdr, BlobRegions::Enumerations);

        let num_entries = little_endian_2_bytes(&common_hdr[0..2]);
		if schema < 4 {
        	let mut hdr = [0; 4];
        	fp.read_exact(&mut hdr, BlobRegions::Enumerations);
        	let max_str_len = little_endian_2_bytes(&hdr[0..2]);
        	let font_family = hdr[2];
        	let idx_entry_len = hdr[3];

        	if root_font_family != font_family {
            	panic!("Mis-match font_family");
        	}
        	Self::validate_schema(schema, idx_entry_len, max_str_len);
		} else {
        	let mut hdr = [0; 1];
        	fp.read_exact(&mut hdr, BlobRegions::Enumerations);
        	let idx_entry_len = hdr[0];
        	Self::validate_schema(schema, idx_entry_len, 256);
		}

        let mut enumerations = HashMap::new();

        for _i in 0..num_entries {
            let (enumeration, entry) = match schema {
                2 => EnumerationsIndexEntry::load_v2(fp),
                3 => EnumerationsIndexEntry::load_v3(fp),
                4 => EnumerationsIndexEntry::load_v3(fp),
                _ => panic!("Invalid schema"),
            };
            let old = enumerations.insert(enumeration, entry);
            if old != None {
                panic!("Two entries with same mnemonic!");
            }
        }
        EnumerationsIndex { enumerations }
    }

    fn validate_schema(schema: u16, idx_entry_len: u8, max_str_len: u16) {
		let mut req_string_len = 16;
        match schema {
            2 => {
                if idx_entry_len != 6 {
                    panic!("V2 EnumerationIndexEntry wrong size 4 != {}", idx_entry_len)
                }
            }
            3 => {
                if idx_entry_len != 5 {
                    panic!("V3 EnumerationIndexEntry wrong size 3 != {}", idx_entry_len)
                }
            }
            4 => {
                if idx_entry_len != 5 {
                    panic!("V3 EnumerationIndexEntry wrong size 3 != {}", idx_entry_len)
                }
				req_string_len = 256;
            }
            _ => panic!("Invalid format"),
        };
        if max_str_len != req_string_len {
            panic!("Max string len should be {} was {}", req_string_len, max_str_len);
        }
    }
}

impl IntoIterator for &EnumerationsIndex {
    type Item = (u16, EnumerationsIndexEntry);
    type IntoIter = EnumerationsIndexIterator;

    fn into_iter(self) -> Self::IntoIter {
        let mut keys = Vec::new();
        for key in self.enumerations.keys() {
            keys.push(*key)
        }
        keys.sort();
        keys.reverse();
        let mut items = Vec::new();
        for key in keys {
            items.push((key, self.enumerations[&key].clone()));
        }
        EnumerationsIndexIterator { items }
    }
}

impl EnumerationsIndexEntry {
    pub fn get_caption_off(&self) -> u32 {
        self.caption_off
    }

    pub fn to_string(&self) -> Result<String, String> {
        match self.blob.get_string(self.caption_off, 16) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("Blob offset {} \n\t {}", self.caption_off, x)),
        }
    }

    fn load_v2(fp: &mut FileBlob) -> (u16, EnumerationsIndexEntry) {
        let mut buf = [0; 6];
        fp.read_exact(&mut buf, BlobRegions::Enumerations);
        let enumeration = little_endian_2_bytes(&buf[0..2]);
        let offset = little_endian_4_bytes(&buf[2..6]);
        if offset == 0 {
            panic! {"Empty slot"};
        };
        let entry = EnumerationsIndexEntry {
            caption_off: offset,
            blob: fp.freeze(),
        };
        (enumeration, entry)
    }

    fn load_v3(fp: &mut FileBlob) -> (u16, EnumerationsIndexEntry) {
        let mut buf = [0; 5];
        fp.read_exact(&mut buf, BlobRegions::Enumerations);
        let enumeration = little_endian_2_bytes(&buf[0..2]);
        let offset = little_endian_3_bytes(&buf[2..5]);
        if offset == 0 {
            panic! {"Empty slot"};
        };
        let entry = EnumerationsIndexEntry {
            caption_off: offset,
            blob: fp.freeze(),
        };
        (enumeration, entry)
    }
}

impl PartialEq for EnumerationsIndexEntry {
    fn eq(&self, other: &Self) -> bool {
        self.caption_off == other.caption_off
    }
}

impl Clone for EnumerationsIndexEntry {
    fn clone(&self) -> EnumerationsIndexEntry {
        EnumerationsIndexEntry {
            caption_off: self.caption_off,
            blob: self.blob.clone(),
        }
    }
}

impl Iterator for EnumerationsIndexIterator {
    type Item = (u16, EnumerationsIndexEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}
