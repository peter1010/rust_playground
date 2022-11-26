use std::collections::HashMap;

use crate::conversion::{little_endian_2_bytes, little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::{FileBlob, RawBlob, BlobRegions};

pub struct ParameterIndex {
    params: HashMap<u8, ParameterIndexEntry>,
}

pub struct ParameterIndexEntry {
    caption_off: u32,
    tooltip_off: u32,
    blob: RawBlob,
}

pub struct ParameterIndexIterator {
    items: Vec<(u8, ParameterIndexEntry)>,
}

impl ParameterIndex {
    ///
    /// V2 does not have menus, all parameters are together
    /// So read all parameters, create parameter indexes (as if we were V3 format)
    /// And return a parameter index per menu
    ///
    pub fn read_v2_entries(
        fp: &mut FileBlob,
        num_entries: u16,
    ) -> HashMap<u8, ParameterIndex> {
        let mut tmp_menus = HashMap::<u8, ParameterIndex>::new();

        for _i in 0..num_entries {
            let (menu, param, entry) = ParameterIndexEntry::load_v2(fp);
            match tmp_menus.get_mut(&menu) {
                None => {
                    let params = HashMap::<u8, ParameterIndexEntry>::new();
                    let mut new = ParameterIndex { params };
                    new.params.insert(param, entry);
                    tmp_menus.insert(menu, new);
                }
                Some(item) => {
                    let old = item.params.insert(param, entry);
                    if old != None {
                        panic!("Two entries with same param!");
                    }
                }
            };
        }
        tmp_menus
    }

    ///
    /// Read and create a V3 ParameterIndex. Also
    /// check and remove parameter 255 which is a placeholder
    /// for menu caption Id
    ///
    pub fn from_v3(
        fp: &mut FileBlob,
        root_font_family: u8,
    ) -> (ParameterIndex, u32, u32) {
        let mut header = [0; 6];
        fp.read_exact(&mut header, BlobRegions::Parameters);

        let num_entries = little_endian_2_bytes(&header[0..2]);
        let max_str_len = little_endian_2_bytes(&header[2..4]);
        let font_family = header[4];
        let idx_entry_len = header[5];

        if root_font_family != font_family {
            panic!("Mis-match font_family");
        }
        let mut params = HashMap::new();

        if idx_entry_len != 0 {
            Self::validate_schema(3, idx_entry_len, max_str_len);

            for _i in 0..num_entries {
                let (param, entry) = ParameterIndexEntry::load_v3(fp);
                let old = params.insert(param, entry);
                if old != None {
                    panic!("Two entries with same parameter!");
                }
            }

            let (caption_off, tooltip_off) = Self::check_param255(&mut params);
            let param_index = ParameterIndex { params };
            (param_index, caption_off, tooltip_off)
        } else {
            (ParameterIndex { params }, 0, 0)
        }
    }

    ///
    /// Read and create a V4 ParameterIndex.
    ///
    pub fn from_v4(fp: &mut FileBlob) -> ParameterIndex {
        let num_entries = fp.read_le_2bytes(BlobRegions::Parameters);
        let idx_entry_len = fp.read_byte(BlobRegions::Parameters);

//		println!("Number of entries {} size {}", num_entries, idx_entry_len);

        let mut params = HashMap::new();

        if idx_entry_len != 0 {
            Self::validate_schema(4, idx_entry_len, 256);

            for _i in 0..num_entries {
                let (param, entry) = ParameterIndexEntry::load_v4(fp);
//				println!("{}", param);

                let old = params.insert(param, entry);
                if old != None {
                    panic!("Two entries with same parameter! param={}", param);
                }
            }

            ParameterIndex { params }
        } else {
            ParameterIndex { params }
        }
    }


    ///
    /// Parameter 255 is a fake parameter used to hold menu caption & tooltip
    ///
    fn check_param255(params: &mut HashMap<u8, ParameterIndexEntry>) -> (u32, u32) {
        let fake_param = match params.remove(&255) {
            Some(param) => param,
            None => return (0, 0),
        };
        return (fake_param.caption_off, fake_param.tooltip_off);
    }

    pub fn self_check_param255(&mut self) -> (u32, u32) {
        ParameterIndex::check_param255(&mut self.params)
    }

    pub fn validate_schema(schema: u16, idx_entry_len: u8, max_str_len: u16) {
		let mut req_str_len = 32;
        match schema {
            2 => {
                if idx_entry_len != 6 {
                    panic!("V2 ParamIndexEntry wrong size 4 != {}", idx_entry_len)
                }
            }
            3 => {
                if idx_entry_len != 5 {
                    panic!("V3 ParamIndexEntry wrong size 3 != {}", idx_entry_len)
                }
            }
            4 => {
                if idx_entry_len != 5 {
                    panic!("V4 ParamIndexEntry wrong size 3 != {}", idx_entry_len)
                }
				req_str_len = 256;
            }
            _ => panic!("Invalid format"),
        };
        if max_str_len != req_str_len {
            panic!("Incorrect string len {} != {}", req_str_len, max_str_len);
        }
    }

    pub fn get_num_params(&self) -> usize {
        self.params.len()
    }
}

impl IntoIterator for &ParameterIndex {
    type Item = (u8, ParameterIndexEntry);
    type IntoIter = ParameterIndexIterator;

    fn into_iter(self) -> Self::IntoIter {
        let mut keys = Vec::new();
        for key in self.params.keys() {
            keys.push(*key)
        }
        keys.sort();
        keys.reverse();
        let mut items = Vec::new();
        for key in keys {
            items.push((key, self.params[&key].clone()));
        }
        ParameterIndexIterator { items }
    }
}

impl ParameterIndexEntry {
    fn load_v4(fp: &mut FileBlob) -> (u8, ParameterIndexEntry) {
        let param = fp.read_byte(BlobRegions::Products);
        let caption_off = fp.read_le_3bytes(BlobRegions::Products);
        let tooltip_off = fp.read_le_3bytes(BlobRegions::Products);
		let mnemonic_off = fp.read_le_3bytes(BlobRegions::Products);

//		println!("{} => {} {} {}", param, caption_off, tooltip_off, mnemonic_off);

        if caption_off == 0 {
            println!("Empty parameter?");
        };
        let param_entry = ParameterIndexEntry {
            caption_off: caption_off,
            tooltip_off: tooltip_off,
            blob: fp.freeze(),
        };
        (param, param_entry)
    }


    fn load_v3(fp: &mut FileBlob) -> (u8, ParameterIndexEntry) {
        let mut buf = [0; 5];
        fp.read_exact(&mut buf, BlobRegions::Products);
        let param = buf[0];
        if buf[1] != 0 {
            panic!("Out of range param {}", buf[0]);
        };
        let offset = little_endian_3_bytes(&buf[2..5]);
        if offset == 0 {
            println!("Empty slot");
        };
        let param_entry = ParameterIndexEntry {
            caption_off: offset,
            tooltip_off: 0,
            blob: fp.freeze(),
        };
        (param, param_entry)
    }

    fn load_v2(fp: &mut FileBlob) -> (u8, u8, ParameterIndexEntry) {
        let mut buf = [0; 6];
        fp.read_exact(&mut buf, BlobRegions::Products);
        let param = buf[0];
        let menu = buf[1];
        let offset = little_endian_4_bytes(&buf[2..6]);
        let param_entry = ParameterIndexEntry {
            caption_off: offset,
            tooltip_off: 0,
            blob: fp.freeze(),
        };
        (menu, param, param_entry)
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

impl PartialEq for ParameterIndexEntry {
    fn eq(&self, other: &Self) -> bool {
        self.caption_off == other.caption_off
    }
}

impl Clone for ParameterIndexEntry {
    fn clone(&self) -> ParameterIndexEntry {
        ParameterIndexEntry {
            caption_off: self.caption_off,
            tooltip_off: self.tooltip_off,
            blob: self.blob.clone(),
        }
    }
}

impl Iterator for ParameterIndexIterator {
    type Item = (u8, ParameterIndexEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}
