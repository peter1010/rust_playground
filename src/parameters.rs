use std::io;
use std::collections::HashMap;

use crate::conversion::{
    little_endian_2_bytes, 
    little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::{FileBlob, RawBlob};


pub struct ParameterIndex {
    params : HashMap<u8, ParameterIndexEntry>,
    max_str_len : u16
}

struct ParameterIndexEntry {
    caption_off : u32,
    tooltip_off : u32,
    blob : RawBlob
}



impl ParameterIndex {


    ///
    /// V2 does not have menus, all parameters are together
    /// So read all parameters, create parameter indexes (as if we were V3 format)
    /// And return a parameter index per menu
    ///
    pub fn read_v2_entries(fp : & mut FileBlob, num_entries : u16, max_str_len : u16) 
            -> io::Result<HashMap::<u8, ParameterIndex>>
    {
        let mut tmp_menus = HashMap::<u8, ParameterIndex>::new();

        for _i in 0..num_entries {
            let (menu, param, entry) = ParameterIndexEntry::load_v2(fp) ?;
            match tmp_menus.get_mut(&menu) {
                None => { 
                    let params = HashMap::<u8, ParameterIndexEntry>::new();
                    let mut new = ParameterIndex { params, max_str_len} ;
                    new.params.insert(param, entry);
                    tmp_menus.insert(menu, new);
                },
                Some(item) => {
                    let old = item.params.insert(param, entry);
                    if old != None {
                        panic!("Two entries with same param!");
                    }
                }
            };
        }
        return Result::Ok(tmp_menus);
    }

    pub fn from(fp : & mut FileBlob, schema : u16, root_font_family : u8) -> io::Result<(ParameterIndex,u32,u32)> 
    {
        let mut header = [0; 6];
        fp.read_exact(& mut header) ?;

        let num_entries = little_endian_2_bytes(&header[0..2]);
        let max_str_len = little_endian_2_bytes(&header[2..4]);
        let font_family = header[4];
        let idx_entry_len = header[5];

        if root_font_family != font_family {
            panic!("Mis-match font_family");
        }
        let mut params = HashMap::new();
        
        if idx_entry_len != 0 {
            println!("- - max str len {}", max_str_len);
            Self::validate_schema(schema, idx_entry_len); 


            for _i in 0..num_entries {
                let (param, entry) = ParameterIndexEntry::load_v3(fp) ?;
                let old = params.insert(param, entry);
                if old != None {
                    panic!("Two entries with same parameter!");
                }
            } 

            let (caption_off, tooltip_off) = Self::check_param255(& mut params);
            let param_index = ParameterIndex { params, max_str_len };
            println!("- - - - params {}", param_index.param_list_as_string());
            Result::Ok((param_index, caption_off, tooltip_off))
        } else {
            Result::Ok((ParameterIndex { params, max_str_len }, 0, 0))
        }
    }
 
    ///
    /// Parameter 255 is a fake parameter used to hold menu caption & tooltip
    ///
    fn check_param255(params : & mut HashMap<u8, ParameterIndexEntry>) -> (u32,u32)
    {
        let fake_param = match params.remove(&255) {
            Some(param) => param,
            None => return (0,0)
        };
        return (fake_param.caption_off, fake_param.tooltip_off);
    }

    pub fn self_check_param255(& mut self) -> (u32, u32)
    {
        ParameterIndex::check_param255(& mut self.params)
    }

    pub fn validate_schema(schema : u16, idx_entry_len : u8) 
    {
        match schema {
            2 => if idx_entry_len != 6 { panic!("V2 ParamIndexEntry wrong size 4 != {}", idx_entry_len) },
            3 => if idx_entry_len != 5 { panic!("V3 ParamIndexEntry wrong size 3 != {}", idx_entry_len) },
            _ => panic!("Invalid format")
        };
    }

    pub fn param_list_as_string(&self) -> String
    {
        let mut params = Vec::new();
        for i in self.params.keys(){
            params.push(i);
        }
        params.sort();

        if params.len() == 0 {
            return String::from("None");
        }
        let mut temp = String::new();
        let mut start = *params[0];
        let mut end = start;
        for i in 1..params.len() {
            let n = *params[i];
            if n == end + 1 {
                end = n;
            } else {
                if end > start {
                    temp = format!("{}{} - {}, ", &temp, start, end);
                } else {
                    temp = format!("{}{}, ", &temp, start);
                }
                start = n;
                end = start;
            }
        }
        if end > start {
            temp = format!("{}{} - {}, ", &temp, start, end);
        } else {
            temp = format!("{}{}, ", &temp, start);
        }
        return temp;
    }

    pub fn get_num_params(&self) -> usize
    {
        self.params.len()
    }

    pub fn display(&self) 
    {
        println!("- - - params {}", self.param_list_as_string());
    }
}


impl ParameterIndexEntry {

    fn load_v3(fp : & mut FileBlob) -> io::Result<(u8, ParameterIndexEntry)>
    {
        let mut buf = [0; 5];
        fp.read_exact(& mut buf) ?;
        let param = buf[0];
        if buf[1] != 0 { 
            panic!("Out of range param {}", buf[0]); 
        };
        let offset = little_endian_3_bytes(&buf[2..5]);
        if offset == 0 { 
            println!("Empty slot");
        };
        let param_entry = ParameterIndexEntry { caption_off : offset, tooltip_off : 0 , blob : fp.freeze()};
        Result::Ok((param, param_entry))
    }

    fn load_v2(fp : & mut FileBlob) -> io::Result<(u8, u8, ParameterIndexEntry)>
    {
        let mut buf = [0; 6];
        fp.read_exact(& mut buf) ?;
        let param = buf[0];
        let menu = buf[1];
        let offset = little_endian_4_bytes(&buf[2..6]);
        let param_entry = ParameterIndexEntry { caption_off : offset, tooltip_off : 0, blob : fp.freeze()};
        Result::Ok((menu, param, param_entry))
    }
}


impl PartialEq for ParameterIndexEntry {

    fn eq(&self, other : & Self) -> bool
    {
        self.caption_off == other.caption_off
    }
}
