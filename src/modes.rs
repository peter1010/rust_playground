use std::collections::HashMap;
use std::io;
use std::rc::Rc;

use crate::conversion::{little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::FileBlob;
use crate::menus::MenuIndex;

pub struct ModeIndex {
    modes: HashMap<u8, ModeIndexEntry>,
}

pub struct ModeIndexEntry {
    menu_index: Rc<MenuIndex>,
}

pub struct ModeIndexIterator {
    items: Vec<(u8, ModeIndexEntry)>,
}

impl ModeIndex {
    pub fn from(fp: &mut FileBlob, schema: u16, font_family: u8) -> io::Result<ModeIndex> {
        let mut header = [0; 2];
        fp.read_exact(&mut header)?;

        let num_modes = header[0];
        let idx_entry_len = header[1];

        Self::validate_schema(schema, idx_entry_len, num_modes);

        let tmp_info = match schema {
            2 => Self::read_v2_entries(fp, num_modes)?,
            3 => Self::read_v3_entries(fp, num_modes)?,
            _ => panic!("Invalid format"),
        };

        //        if tmp_info.len() > 1 {
        //            println!("- Number of modes = {}", tmp_info.len());
        //        }

        let mut modes = HashMap::new();

        match schema {
            2 => {
                for (mode_num, offset) in tmp_info {
                    if offset != 0 {
                        fp.set_pos(offset);
                        let menu_index = MenuIndex::from_v2(fp, font_family)?;
                        modes.insert(
                            mode_num,
                            ModeIndexEntry {
                                menu_index: Rc::<MenuIndex>::new(menu_index),
                            },
                        );
                    }
                }
            }
            3 => {
                for (mode_num, offset) in tmp_info {
                    if offset != 0 {
                        fp.set_pos(offset);
                        let menu_index = MenuIndex::from_v3plus(fp, font_family)?;
                        modes.insert(
                            mode_num,
                            ModeIndexEntry {
                                menu_index: Rc::<MenuIndex>::new(menu_index),
                            },
                        );
                    }
                }
            }
            _ => panic!("Invalid format"),
        };

        Result::Ok(ModeIndex { modes })
    }

    pub fn get_num_modes(&self) -> usize {
        self.modes.len()
    }

    fn validate_schema(schema: u16, idx_entry_len: u8, num_modes: u8) {
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
            _ => panic!("Invalid format"),
        };
        if num_modes > 4 {
            panic!("Too many modes");
        }
    }

    fn read_v2_entries(fp: &mut FileBlob, num_entries: u8) -> io::Result<Vec<(u8, u32)>> {
        let mut tmp_info = Vec::new();

        for i in 0..num_entries {
            let mut buf = [0; 5];
            fp.read_exact(&mut buf)?;
            let mode_num = buf[0];
            if num_entries > 1 {
                if mode_num != i + 1 {
                    panic!("Out of seq mode numbers {} != {}", mode_num, i);
                }
            } else if mode_num != 0 && mode_num != 1 {
                panic!("Invalid mode_num {}", mode_num);
            }
            let offset = little_endian_4_bytes(&buf[1..5]);
            if offset == 0 {
                panic!("Offset is zero")
            };
            tmp_info.push((mode_num, offset))
        }
        return Result::Ok(tmp_info);
    }

    fn read_v3_entries(fp: &mut FileBlob, num_entries: u8) -> io::Result<Vec<(u8, u32)>> {
        let mut tmp_info = Vec::new();

        for i in 0..num_entries {
            let mut buf = [0; 3];
            fp.read_exact(&mut buf)?;
            let offset = little_endian_3_bytes(&buf[0..3]);
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
        return Result::Ok(tmp_info);
    }
}

impl IntoIterator for &ModeIndex {
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

impl ModeIndexEntry {
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
