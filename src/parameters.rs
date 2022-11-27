use std::collections::{HashMap, HashSet};

use crate::blob::{FileBlob, RawBlob, BlobRegions};
use crate::mnemonics::MnemonicIndex;
use std::rc::Rc;

pub struct ParameterIndex {
    params: HashMap<u8, ParameterIndexEntry>,
}

pub struct ParameterIndexEntry {
    param_num: u8,
    caption_off: u32,
    tooltip_off: u32,
    mnemonic: Rc<MnemonicIndex>,
    blob: RawBlob,
}

pub struct ParameterIndexIterator {
    items: Vec<(u8, ParameterIndexEntry)>,
}

impl ParameterIndex {

    pub fn new(params: HashMap<u8, ParameterIndexEntry>) -> ParameterIndex
    {
        let mut hits = HashSet::<u8>::new();

        for entry in &params {
            let param_num = entry.1.param_num;

            assert_eq!(*entry.0, param_num);

            if hits.contains(&param_num) {
                panic!("Duplicate parameter number found");
            }
            hits.insert(param_num);
        }
        ParameterIndex { params }
    }

    ///
    /// V2 does not have menus, all parameters are together
    /// So read all parameters, create parameter indexes (as if we were V3 format)
    /// And return a parameter index per menu
    ///
    pub fn read_v2_entries(fp: &mut FileBlob, num_entries: u16) -> HashMap<u8, ParameterIndex> 
    {
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
                    item.params.insert(param, entry);
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
    pub fn from_v3(fp: &mut FileBlob, root_font_family: u8) -> (ParameterIndex, u32, u32) {
        let num_entries = fp.read_le_2bytes(BlobRegions::Parameters);
        let max_str_len = fp.read_le_2bytes(BlobRegions::Parameters);
        let font_family = fp.read_byte(BlobRegions::Parameters);
        let idx_entry_len = fp.read_byte(BlobRegions::Parameters);

        if root_font_family != font_family {
            panic!("Mis-match font_family");
        }
        let mut params = HashMap::new();

        if idx_entry_len != 0 {
            Self::validate_schema(3, idx_entry_len, max_str_len);

            for _i in 0..num_entries {
                let (param, entry) = ParameterIndexEntry::load_v3(fp);
                params.insert(param, entry);
            }

            let (caption_off, tooltip_off) = Self::check_param255(&mut params);
            let param_index = ParameterIndex { params };
            (param_index, caption_off, tooltip_off)
        } else {
            (ParameterIndex::new(params), 0, 0)
        }
    }

    ///
    /// Read and create a V4 ParameterIndex.
    ///
    pub fn from_v4(fp: &mut FileBlob) -> ParameterIndex {
        let num_params = fp.read_byte(BlobRegions::Parameters);
        let idx_entry_len = fp.read_byte(BlobRegions::Parameters);

//		println!("Number of entries {} size {}", num_entries, idx_entry_len);

        let mut params = HashMap::new();
        

        if idx_entry_len != 0 {
            Self::validate_schema(4, idx_entry_len, 256);

            let tmp_info = Self::read_v4_entries(fp, num_params);

            for (param, caption_off, tooltip_off, mnemonic_off) in tmp_info {
//			    println!("{} => {}", menu, offset);

                let mnemonic = if mnemonic_off > 0 {
                    fp.set_pos(mnemonic_off);
                    MnemonicIndex::from(fp)
                } else {
                    MnemonicIndex::empty()
                };

//				println!("{}", param);

                params.insert(param, ParameterIndexEntry::new(
                    param, caption_off, tooltip_off,
                    mnemonic, fp));
            }

            ParameterIndex::new(params)
        } else {
            ParameterIndex::new(params)
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
    
    fn read_v4_entries(fp: &mut FileBlob, num_entries: u8) -> Vec<(u8, u32, u32, u32)> {
        let mut tmp_info = Vec::new();

        for _i in 0..num_entries {
            let param = fp.read_byte(BlobRegions::Parameters);
            let caption_off = fp.read_le_3bytes(BlobRegions::Menus);
            let tooltip_off = fp.read_le_3bytes(BlobRegions::Menus);
            let mnemonic_off = fp.read_le_3bytes(BlobRegions::Menus);
            if caption_off > 0 {
                tmp_info.push((param, caption_off, tooltip_off, mnemonic_off));
            }
        }
        tmp_info
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

    fn new(param_num: u8, caption_off :u32, tooltip_off:u32, mnemonic : MnemonicIndex, fp : & mut FileBlob)
    -> ParameterIndexEntry
    {
        ParameterIndexEntry {
            param_num,
            caption_off: caption_off,
            tooltip_off: tooltip_off,
            mnemonic : Rc::new(mnemonic),
            blob: fp.freeze()
        }
    }

    fn load_v3(fp: &mut FileBlob) -> (u8, ParameterIndexEntry) {
        let param = fp.read_le_2bytes(BlobRegions::Parameters);
        if param > 255  {
            panic!("Out of range param {}", param);
        };
        let offset = fp.read_le_3bytes(BlobRegions::Parameters);
        if offset == 0 {
            println!("Empty slot");
        };
        let param_entry = ParameterIndexEntry::new(
            param as u8, offset, 0,
            MnemonicIndex::empty(), fp
        );
        (param as u8, param_entry)
    }

    fn load_v2(fp: &mut FileBlob) -> (u8, u8, ParameterIndexEntry) {
        let param = fp.read_byte(BlobRegions::Parameters);
        let menu = fp.read_byte(BlobRegions::Parameters);
        let offset = fp.read_le_4bytes(BlobRegions::Parameters);
        let param_entry = ParameterIndexEntry::new(
            param, offset, 0,
            MnemonicIndex::empty(),
            fp
        );
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

    pub fn get_mnemonics(&self) -> &MnemonicIndex
    {
        &self.mnemonic
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
            param_num: self.param_num,
            caption_off: self.caption_off,
            tooltip_off: self.tooltip_off,
            mnemonic: self.mnemonic.clone(),
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
