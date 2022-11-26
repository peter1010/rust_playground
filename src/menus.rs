use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::blob::{FileBlob, RawBlob, BlobRegions};
use crate::parameters::ParameterIndex;

pub struct MenuIndex 
{
    menus: HashMap<u8, MenuIndexEntry>,
}

pub struct MenuIndexEntry 
{
    menu_num : u8,
    caption_off: u32,
    tooltip_off: u32,
    param_index: Rc<ParameterIndex>,
    blob: RawBlob,
}

pub struct MenuIndexIterator
{
    items: Vec<(u8, MenuIndexEntry)>,
}

impl MenuIndex {

    pub fn new(menus : HashMap<u8, MenuIndexEntry>) -> MenuIndex
    {
        let mut hits = HashSet::<u8>::new();

        for entry in &menus {
            let menu_num = entry.1.menu_num;

            assert_eq!(*entry.0, menu_num);
            
            if hits.contains(&menu_num) {
                panic!("Duplicate menus detected");
            }
            hits.insert(menu_num);
        }
        MenuIndex { menus }
    }

    ///
    /// V2 format does not have a MenuIndex, So create an pseudo one
    ///
    pub fn from_v2(fp: &mut FileBlob, root_font_family: u8) -> MenuIndex {
        // V2 there are no menu Indexes!
        // Read ParameterIndex

        let num_entries = fp.read_le_2bytes(BlobRegions::Parameters);
        let max_str_len = fp.read_le_2bytes(BlobRegions::Parameters);
        let font_family = fp.read_byte(BlobRegions::Parameters);
        let idx_entry_len = fp.read_byte(BlobRegions::Parameters);

        if root_font_family != font_family {
            panic!("Mis-match font_family");
        }

        ParameterIndex::validate_schema(2, idx_entry_len, max_str_len);

        // Create menus anyway...
        let tmp_menus = ParameterIndex::read_v2_entries(fp, num_entries);

        let mut menus = HashMap::<u8, MenuIndexEntry>::new();

        for entry in tmp_menus {
            let menu_num = entry.0;
            let mut param_index = entry.1;

            let (caption_off, tooltip_off) = param_index.self_check_param255();
                //                let temp = param_index.param_list_as_string();
                //              println!("- - Menu {}", menu);
                //                println!("- - - Params {}", temp);

            menus.insert(
                menu_num,
                MenuIndexEntry::new(
                        menu_num,
                        caption_off,
                        tooltip_off,
                        param_index,
                        fp
                    ),
            );
        }

        MenuIndex::new(menus)
    }

    ///
    /// Create a MenuIndex from v3 schema
    ///
    pub fn from_v3(fp: &mut FileBlob, font_family: u8) -> MenuIndex {
        let num_menus = fp.read_byte(BlobRegions::Menus);
        let idx_entry_len = fp.read_byte(BlobRegions::Menus);

        let mut menus = HashMap::new();

        Self::validate_schema(3, idx_entry_len);

        let tmp_info = Self::read_v3_entries(fp, num_menus);

        for (menu_num, offset) in tmp_info {
            fp.set_pos(offset);
            let (param_index, caption_off, tooltip_off) = ParameterIndex::from_v3(fp, font_family);
            let menu_entry = MenuIndexEntry::new(
                menu_num,
                caption_off,
                tooltip_off,
                param_index,
                fp
            );
            menus.insert(menu_num, menu_entry);
        }
        MenuIndex::new(menus)
    }

    ///
    /// Create a MenuIndex from v4 schema
    ///
    pub fn from_v4(fp: &mut FileBlob) -> MenuIndex {
        let num_menus = fp.read_byte(BlobRegions::Menus);
        let idx_entry_len = fp.read_byte(BlobRegions::Menus);

        let mut menus = HashMap::new();

        Self::validate_schema(4, idx_entry_len);

        let tmp_info = Self::read_v4_entries(fp, num_menus);

        for (menu_num, caption_off, tooltip_off, offset) in tmp_info {
//			println!("{} => {}", menu_num, offset);
            fp.set_pos(offset);
            let param_index = ParameterIndex::from_v4(fp);
            let menu_entry = MenuIndexEntry::new(
                menu_num,
                caption_off,
                tooltip_off,
                param_index,
                fp,
            );
            menus.insert(menu_num, menu_entry);
        }
        MenuIndex::new(menus)
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
            let offset = fp.read_le_3bytes(BlobRegions::Menus);
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
            let caption_off = fp.read_le_3bytes(BlobRegions::Menus);
            let tooltip_off = fp.read_le_3bytes(BlobRegions::Menus);
            let offset = fp.read_le_3bytes(BlobRegions::Menus);
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

    pub fn new(menu_num : u8, caption_off : u32, tooltip_off : u32, param_index : ParameterIndex, fp : & mut FileBlob)
    -> MenuIndexEntry
    {
        MenuIndexEntry {
            menu_num,
            caption_off,
            tooltip_off,
            param_index: Rc::<ParameterIndex>::new(param_index),
            blob: fp.freeze(),
        }
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

    pub fn get_params(&self) -> &ParameterIndex {
        &self.param_index
    }
}

impl PartialEq for MenuIndexEntry {
    fn eq(&self, other: &Self) -> bool {
        self.caption_off == other.caption_off
    }
}

impl Clone for MenuIndexEntry 
{
    fn clone(&self) -> MenuIndexEntry {
        MenuIndexEntry {
            menu_num: self.menu_num,
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
