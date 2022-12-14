use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::blob::{FileBlob, BlobRegions};
use crate::menus::MenuIndex;

pub struct ModeIndex
{
    modes: HashMap<u8, ModeIndexEntry>,
}

pub struct ModeIndexEntry 
{
    mode_num: u8,
    menu_index: Rc<MenuIndex>,
}

pub struct ModeIndexIterator 
{
    items: Vec<(u8, ModeIndexEntry)>,
}


impl ModeIndex 
{
    pub fn new(modes: HashMap<u8, ModeIndexEntry>) -> ModeIndex 
    {
        let mut hits = HashSet::new();

        for entry in &modes {
            let mode_num = entry.1.mode_num;

            assert_eq!(*entry.0, mode_num);

            if hits.contains(&mode_num) {
                panic!("Duplicate modes detected");
            }
            hits.insert(mode_num);
        }
        ModeIndex { modes }
    }

    pub fn create_from_file(fp: &mut FileBlob, schema: u16, font_family: u8) -> ModeIndex 
    {
        let num_modes = fp.read_byte(BlobRegions::Modes);
        let idx_entry_len = fp.read_byte(BlobRegions::Modes);

        Self::validate_schema(schema, idx_entry_len, num_modes);

        let tmp_info = match schema {
            2 => Self::read_v2_entries(fp, num_modes),
            3 => Self::read_v3_entries(fp, num_modes),
            4 => Self::read_v3_entries(fp, num_modes),
            _ => panic!("Invalid format"),
        };

        let mut modes = HashMap::new();
        
        for (mode_num, offset) in tmp_info {
            if offset != 0 {
                fp.set_pos(offset);

                let menu_index = match schema {
                    2 => MenuIndex::from_v2(fp, font_family),
                    3 => MenuIndex::from_v3(fp, font_family),
                    4 => MenuIndex::from_v4(fp),
                    _ => panic!("Invalid format")
                };
                modes.insert(
                    mode_num,
                    ModeIndexEntry::new(mode_num, menu_index)
                );
            } else {
                panic!("Unexpected empty mode");
            }
        }
        ModeIndex::new(modes)
    }

    pub fn get_num_modes(&self) -> usize
    {
        self.modes.len()
    }

    fn validate_schema(schema: u16, idx_entry_len: u8, num_modes: u8) 
    {
        match schema {
            2 => {
                if idx_entry_len != 5 {
                    panic!("ModeIndexEntry wrong size 5 != {}", idx_entry_len)
                }
            }
            3 => {
                if idx_entry_len != 3 {
                    panic!("ModeIndexEntry wrong size 3 != {}", idx_entry_len)
                }
            }
            4 => {
                if idx_entry_len != 3 {
                    panic!("ModeIndexEntry wrong size 3 != {}", idx_entry_len)
                }
            }
            _ => panic!("Invalid format"),
        };
        if num_modes < 1 {
            panic!("Too few modes");
        }
        if num_modes > 4 {
            panic!("Too many modes");
        }
    }

    fn read_v2_entries(fp: &mut FileBlob, num_entries: u8) -> Vec<(u8, u32)> {
        let mut tmp_info = Vec::new();

        for i in 0..num_entries {
            let mode_num = fp.read_byte(BlobRegions::Modes);
            if num_entries > 1 {
                if mode_num != i + 1 {
                    panic!("Out of seq mode numbers {} != {}", mode_num, i);
                }
            } else if mode_num != 0 && mode_num != 1 {
                panic!("Invalid mode_num {}", mode_num);
            }
            let offset = fp.read_le_4bytes(BlobRegions::Modes);
            if offset == 0 {
                panic!("Offset is zero")
            };
            tmp_info.push((mode_num, offset))
        }
        tmp_info
    }

    fn read_v3_entries(fp: &mut FileBlob, num_entries: u8) -> Vec<(u8, u32)> {
        let mut tmp_info = Vec::new();

        for i in 0..num_entries {
            let offset = fp.read_le_3bytes(BlobRegions::Modes);
            let mode_num = if num_entries == 1 {
                if offset == 0 {
                    panic!("Offset is zero")
                }
                0
            } else {
                i + 1
            };
            if offset != 0 {
                tmp_info.push((mode_num, offset));
            }
        }
        tmp_info
    }
}

impl IntoIterator for &ModeIndex 
{
    type Item = (u8, ModeIndexEntry);
    type IntoIter = ModeIndexIterator;

    fn into_iter(self) -> Self::IntoIter {
        let mut keys = Vec::new();
        for key in self.modes.keys() {
            keys.push(*key)
        }
        keys.sort();
        keys.reverse();
        let mut items = Vec::new();
        for key in keys {
            items.push((key, self.modes[&key].clone()));
        }
        ModeIndexIterator { items }
    }
}

impl ModeIndexEntry 
{
    pub fn new(mode_num : u8, menu_index : MenuIndex) -> ModeIndexEntry
    {
        ModeIndexEntry
        {
            mode_num,
            menu_index: Rc::<MenuIndex>::new(menu_index),
        }
    }

    pub fn to_string(&self, mode: u8) -> Result<String, String> {
        Result::Ok(format!(
            "Mode '{}' num of menus = {}",
            match mode {
                0 => "Any",
                1 => "Open Loop",
                2 => "RFC-A",
                3 => "RFC-S",
                4 => "Regen",
                _ => panic!("Unknown mode"),
            },
            self.menu_index.get_num_menus()
        ))
    }

    pub fn get_menus(&self) -> &MenuIndex {
        &self.menu_index
    }
}

impl Clone for ModeIndexEntry {
    fn clone(&self) -> ModeIndexEntry {
        ModeIndexEntry {
            mode_num : self.mode_num,
            menu_index: self.menu_index.clone(),
        }
    }
}

impl Iterator for ModeIndexIterator {
    type Item = (u8, ModeIndexEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}
