use std::collections::HashMap;
use std::rc::Rc;

use crate::conversion::{little_endian_2_bytes, little_endian_3_bytes};

use crate::blob::{FileBlob, RawBlob, BlobRegions};
use crate::parameters::ParameterIndex;

pub struct MenuIndex {
    menus: HashMap<u8, MenuIndexEntry>,
}

pub struct MenuIndexEntry {
    caption_off: u32,
    tooltip_off: u32,
    param_index: Rc<ParameterIndex>,
    blob: RawBlob,
}

pub struct MenuIndexIterator {
    items: Vec<(u8, MenuIndexEntry)>,
}

impl MenuIndex {
    ///
    /// V2 format does not have a MenuIndex, So create an pseudo one
    ///
    pub fn from_v2(fp: &mut FileBlob, root_font_family: u8) -> MenuIndex {
        // V2 there are no menu Indexes!
        // Read ParameterIndex

        let mut header = [0; 6];
        fp.read_exact(&mut header, BlobRegions::Parameters);

        let num_entries = little_endian_2_bytes(&header[0..2]);
        let max_str_len = little_endian_2_bytes(&header[2..4]);
        let font_family = header[4];
        let idx_entry_len = header[5];

        if root_font_family != font_family {
            panic!("Mis-match font_family");
        }

        ParameterIndex::validate_schema(2, idx_entry_len, max_str_len);

        // Create menus anyway...
        let mut tmp_menus = ParameterIndex::read_v2_entries(fp, num_entries);

        let mut menus = HashMap::new();
        let mut keys: Vec<u8> = tmp_menus.keys().cloned().collect();
        keys.sort();

        for menu in keys {
            if let Some(mut param_index) = tmp_menus.remove(&menu) {
                let (caption_off, tooltip_off) = param_index.self_check_param255();
                //                let temp = param_index.param_list_as_string();
                //              println!("- - Menu {}", menu);
                //                println!("- - - Params {}", temp);

                menus.insert(
                    menu,
                    MenuIndexEntry {
                        caption_off,
                        tooltip_off,
                        param_index: Rc::<ParameterIndex>::new(param_index),
                        blob: fp.freeze(),
                    },
                );
            }
        }

        MenuIndex { menus }
    }

    ///
    /// Create a MenuIndex from v3 schema
    ///
    pub fn from_v3(fp: &mut FileBlob, font_family: u8) -> MenuIndex {
        let mut header = [0; 2];
        fp.read_exact(&mut header, BlobRegions::Menus);

        let num_menus = header[0];
        let idx_entry_len = header[1];

        let mut menus = HashMap::new();

        Self::validate_schema(3, idx_entry_len);

        let tmp_info = Self::read_v3_entries(fp, num_menus);

        for (menu, offset) in tmp_info {
            fp.set_pos(offset);
            let (param_index, caption_off, tooltip_off) = ParameterIndex::from_v3(fp, font_family);
            let menu_entry = MenuIndexEntry {
                caption_off,
                tooltip_off,
                param_index: Rc::<ParameterIndex>::new(param_index),
                blob: fp.freeze(),
            };
            let old = menus.insert(menu, menu_entry);
            if old != None {
                panic!("Duplicate menus found");
            }
        }
        MenuIndex { menus }
    }

    ///
    /// Create a MenuIndex from v4 schema
    ///
    pub fn from_v4(fp: &mut FileBlob) -> MenuIndex {
        let mut header = [0; 2];
        fp.read_exact(&mut header, BlobRegions::Menus);

        let num_menus = header[0];
        let idx_entry_len = header[1];

        let mut menus = HashMap::new();

        Self::validate_schema(4, idx_entry_len);

        let tmp_info = Self::read_v4_entries(fp, num_menus);

        for (menu, caption_off, tooltip_off, offset) in tmp_info {
//			println!("{} => {}", menu, offset);
            fp.set_pos(offset);
            let param_index = ParameterIndex::from_v4(fp);
            let menu_entry = MenuIndexEntry {
                caption_off,
                tooltip_off,
                param_index: Rc::<ParameterIndex>::new(param_index),
                blob: fp.freeze(),
            };
            let old = menus.insert(menu, menu_entry);
            if old != None {
                panic!("Duplicate menus found");
            }
        }
        MenuIndex { menus }
    }


    fn validate_schema(schema: u16, idx_entry_len: u8) {
        match schema {
            2 => {
                if idx_entry_len != 6 {
                    panic!("V2 ParamIndexEntry wrong size 6 != {}", idx_entry_len)
                }
            }
            3 => {
                if idx_entry_len != 3 {
                    panic!("V3 MenuIndexEntry wrong size 3 != {}", idx_entry_len)
                }
            }
            4 => {
                if idx_entry_len != 9 {
                    panic!("V4 MenuIndexEntry wrong size 9 != {}", idx_entry_len)
                }
            }
            _ => panic!("Invalid format"),
        };
    }

    ///
    /// Read and return a temp list of V3 menu entries
    ///
    fn read_v3_entries(fp: &mut FileBlob, num_entries: u8) -> Vec<(u8, u32)> {
        let mut tmp_info = Vec::new();

        for i in 0..num_entries {
            let mut buf = [0; 3];
            fp.read_exact(&mut buf, BlobRegions::Menus);
            let offset = little_endian_3_bytes(&buf[0..3]);
            if offset > 0 {
                tmp_info.push((i, offset));
            }
        }
        tmp_info
    }

    ///
    /// Read and return a temp list of V4 menu entries
    ///
    fn read_v4_entries(fp: &mut FileBlob, num_entries: u8) -> Vec<(u8, u32, u32, u32)> {
        let mut tmp_info = Vec::new();

        for i in 0..num_entries {
            let mut buf = [0; 9];
            fp.read_exact(&mut buf, BlobRegions::Menus);
            let caption_off = little_endian_3_bytes(&buf[0..3]);
            let tooltip_off = little_endian_3_bytes(&buf[3..6]);
            let offset = little_endian_3_bytes(&buf[6..9]);
            if caption_off > 0 {
                tmp_info.push((i, caption_off, tooltip_off, offset));
            }
        }
        tmp_info
    }


    pub fn get_num_menus(&self) -> usize {
        self.menus.len()
    }
}

impl IntoIterator for &MenuIndex {
    type Item = (u8, MenuIndexEntry);
    type IntoIter = MenuIndexIterator;

    fn into_iter(self) -> Self::IntoIter {
        let mut keys = Vec::new();
        for key in self.menus.keys() {
            keys.push(*key)
        }
        keys.sort();
        keys.reverse();
        let mut items = Vec::new();
        for key in keys {
            items.push((key, self.menus[&key].clone()));
        }
        MenuIndexIterator { items }
    }
}

impl MenuIndexEntry {
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

    pub fn get_params(&self) -> &ParameterIndex {
        &self.param_index
    }
}

impl PartialEq for MenuIndexEntry {
    fn eq(&self, other: &Self) -> bool {
        self.caption_off == other.caption_off
    }
}

impl Clone for MenuIndexEntry {
    fn clone(&self) -> MenuIndexEntry {
        MenuIndexEntry {
            caption_off: self.caption_off,
            tooltip_off: self.tooltip_off,
            param_index: self.param_index.clone(),
            blob: self.blob.clone(),
        }
    }
}

impl Iterator for MenuIndexIterator {
    type Item = (u8, MenuIndexEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}
