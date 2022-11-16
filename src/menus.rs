use std::io;
use std::collections::HashMap;

use crate::conversion::{
    little_endian_2_bytes, 
    little_endian_3_bytes};

use crate::blob::{FileBlob, RawBlob};
use crate::parameters::ParameterIndex;


pub struct MenuIndex {
    menus : HashMap<u8, MenuIndexEntry>
}

struct MenuIndexEntry {
    caption_off : u32,
    tooltip_off : u32,
    param_index : ParameterIndex,
    blob : RawBlob
}


impl MenuIndex {

    ///
    /// V2 format does not have a MenuIndex,
    ///
    pub fn from_v2(fp : & mut FileBlob, root_font_family : u8) -> io::Result<MenuIndex> 
    {
        // V2 there are no menu Indexes!
        // Read ParameterIndex

        let mut header = [0; 6];
        fp.read_exact(& mut header) ?;

        let num_entries = little_endian_2_bytes(&header[0..2]);
        let max_str_len = little_endian_2_bytes(&header[2..4]);
        let font_family = header[4];
        let idx_entry_len = header[5];

        if root_font_family != font_family {
            panic!("Mis-match font_family");
        }

        println!("- - max str len {}", max_str_len);
        ParameterIndex::validate_schema(2, idx_entry_len); 

        // Create menus anyway...
        let mut tmp_menus = ParameterIndex::read_v2_entries(fp, num_entries, max_str_len) ?;
        // -> io::Result<HashMap::<u8, (ParameterIndex)>>
        
        let mut menus = HashMap::new();
        let mut keys : Vec<u8> = tmp_menus.keys().cloned().collect();
        keys.sort();

        for menu in keys {
            if let Some(mut param_index) = tmp_menus.remove(&menu) {
                let (caption_off, tooltip_off) = param_index.self_check_param255();
                let temp = param_index.param_list_as_string();
  //              println!("- - Menu {}", menu);
                println!("- - - Params {}", temp);
 
                menus.insert( menu, MenuIndexEntry { caption_off, tooltip_off, param_index, blob : fp.freeze()});
            }
        }

        Result::Ok(MenuIndex { menus })
    }


 
    pub fn from_v3plus(fp : & mut FileBlob, font_family : u8) -> io::Result<MenuIndex> 
    {
        let mut header = [0; 2];
        fp.read_exact(& mut header) ?;

        let num_menus = header[0];
        let idx_entry_len = header[1];

        let mut menus = HashMap::new();
        
        Self::validate_schema(3, idx_entry_len); 

        let tmp_info = Self::read_v3_entries(fp, num_menus) ?;

        // println!("Num of menus {}", tmp_info.len());


        for (menu, offset) in tmp_info {
//            println!("- - Menu {}", menu);
            fp.set_pos(offset);
            let (param_index, caption_off, tooltip_off) = ParameterIndex::from(fp, 3, font_family) ?;
            let menu_entry = MenuIndexEntry { caption_off, tooltip_off, param_index, blob : fp.freeze() };
            let old = menus.insert(menu, menu_entry);
            if old != None {
                panic!("Duplicate menus found");
            }
        } 
        Result::Ok(MenuIndex { menus })
   }

    fn validate_schema(schema : u16, idx_entry_len : u8) 
    {
        match schema {
            2 => if idx_entry_len != 6 { panic!("V2 ParamIndexEntry wrong size 6 != {}", idx_entry_len) },
            3 => if idx_entry_len != 3 { panic!("V3 MenuIndexEntry wrong size 3 != {}", idx_entry_len) },
            _ => panic!("Invalid format")
        };
    }


    fn read_v3_entries(fp : & mut FileBlob, num_entries : u8) -> io::Result<Vec<(u8, u32)>>
    {
        let mut tmp_info = Vec::new();

        for i in 0..num_entries {
            let mut buf = [0; 3];
            fp.read_exact(& mut buf) ?;
            let offset = little_endian_3_bytes(&buf[0..3]);
            if offset > 0 {
                tmp_info.push((i, offset));
            }
        }
        return Result::Ok(tmp_info);
    }
 
    pub fn get_num_menus(&self) -> usize
    {
        self.menus.len()
    }

    pub fn display(&self)
    {
        println!("- Num of menus = {}", self.menus.len());
    }
}


impl MenuIndexEntry {

//    pub fn display(&self)
//    {
//        println!("- - Menu {} num of params = {}", self.menu, 
//            self.param_index.get_num_params());
//    }
}

impl PartialEq for MenuIndexEntry {

    fn eq(&self, other : & Self) -> bool
    {
        self.caption_off == other.caption_off
    }
}
