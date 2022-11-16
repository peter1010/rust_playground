use std::io;
use std::collections::{HashMap, HashSet};

use crate::conversion::{
    little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::FileBlob;
use crate::menus::MenuIndex;


pub struct ModeIndex {
    modes : HashMap<u8,ModeIndexEntry>
}

pub struct ModeIndexEntry {
    menu_index : MenuIndex
}


impl ModeIndex {

    pub fn from(fp : & mut FileBlob, schema : u16, font_family : u8) -> io::Result<ModeIndex> 
    {
        let mut header = [0; 2];
        fp.read_exact(& mut header) ?;

        let num_modes = header[0];
        let idx_entry_len = header[1];

        Self::validate_schema(schema, idx_entry_len, num_modes);
        
        let tmp_info = match schema {
            2 => Self::read_v2_entries(fp, num_modes, schema) ?,
            3 => Self::read_v3_entries(fp, num_modes, schema) ?,
            _ => panic!("Invalid format")
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
                        let menu_index = MenuIndex::from_v2(fp, font_family) ?;
                        modes.insert( mode_num, ModeIndexEntry { menu_index});
                    }
                }
            },
            3 => {
                for (mode_num, offset) in tmp_info {
                    if offset != 0 {
                        fp.set_pos(offset);
                        let menu_index = MenuIndex::from_v3plus(fp, font_family) ?;
                        modes.insert( mode_num, ModeIndexEntry { menu_index});
                    }
                }
            },
            _ => panic!("Invalid format")
        };

        Result::Ok(ModeIndex { modes })
    }


    fn validate_schema(schema : u16, idx_entry_len : u8, num_modes : u8) 
    {
        match schema {
            2 => if idx_entry_len != 5 { panic!("ModeIndexEntry wrong size 5 != {}", idx_entry_len) },
            3 => if idx_entry_len != 3 { panic!("ModeIndexEntry wrong size 3 != {}", idx_entry_len) },
            _ => panic!("Invalid format")
        };
    }


    fn read_v2_entries(fp : & mut FileBlob, num_entries : u8, schema : u16) -> io::Result<Vec<(u8,u32)>>
    {
        let mut tmp_info = Vec::new();

        for i in 0..num_entries {
            let mut buf = [0; 5];
            fp.read_exact(& mut buf) ?;
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
            tmp_info.push((mode_num,offset))
        }
        return Result::Ok(tmp_info);
    }

    
    fn read_v3_entries(fp : & mut FileBlob, num_entries : u8, schema : u16) -> io::Result<Vec<(u8,u32)>>
    {
        let mut tmp_info = Vec::new();

        for i in 0..num_entries {
            let mut buf = [0; 3];
            fp.read_exact(& mut buf) ?;
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


    fn display(&self)
    {
        println!("Num of Modes = {}", self.modes.len());
    }
}


impl ModeIndexEntry {

    pub fn display(&self)
    {
//        println!("- Mode {} num of menus = {}", match self.mode_num {
//            0 => "Any",
//            1 => "Open Loop",
//            2 => "RFC-A",
//            3 => "RFC-S",
//            4 => "Regen",
//            _ => panic!("Unknown mode")
//        }, self.menu_index.menus.len());
    }
}
