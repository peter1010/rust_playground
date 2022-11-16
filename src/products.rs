use std::fs::File;
use std::io::Read;
use std::io;
use std::collections::{HashMap, HashSet};

use crate::conversion::{
    little_endian_2_bytes, 
    little_endian_2_bytes_as_u8, 
    little_endian_3_bytes, little_endian_4_bytes, little_endian_4_version};

use crate::blob::{FileBlob, RawBlob};
use crate::modes::ModeIndex;


pub struct ProductIndex {
    products : HashMap<u16, ProductIndexEntry>
}

pub struct ProductIndexEntry {
    derivative_id_low : u16,
    derivative_id_high : u16,
    flags : u16,
    mode_index : ModeIndex
}


impl ProductIndex {

    pub fn from(fp : & mut FileBlob, schema : u16, font_family : u8) -> io::Result<ProductIndex> 
    {
        let mut header = [0; 2];
        fp.read_exact(& mut header) ?;

        // Product index header
        let num_products = header[0];
        let idx_entry_len = header[1];

//        println!("Number of products = {}", num_products);

        Self::validate_schema(schema, idx_entry_len);

        let tmp_info = match schema {
            2 => Self::read_v2_entries(fp, num_products, schema) ?,
            3 => Self::read_v3_entries(fp, num_products, schema) ?,
            _ => panic!("Invalid format")
        };

        let mut products = HashMap::new();

        for info in tmp_info {
            let (product_id, derivative_id_low, derivative_id_high, flags, offset) = info;
//            if derivative_id_high > derivative_id_low {
//                println!("Product = {} : {} - {}", product_id, derivative_id_low, derivative_id_high);
//            } else {
//                println!("Product = {} : {}", product_id, derivative_id_low);
//            }

            fp.set_pos(offset);
            let mode_index = ModeIndex::from(fp, schema, font_family) ?;
            products.insert( product_id, ProductIndexEntry { derivative_id_low, derivative_id_high, flags, mode_index});
        }

        Result::Ok(ProductIndex { products })
    }


    fn validate_schema(schema : u16, idx_entry_len : u8)
    {
        match schema {
            2 => if idx_entry_len != 8 { panic!("ProductIndexEntry wrong size 8 != {}", idx_entry_len) },
            3 => if idx_entry_len != 11 { panic!("ProductIndexEntry wrong size 11 != {}", idx_entry_len) },
            _ => panic!("Invalid format")
        };
    }


    fn read_v2_entries(fp : & mut FileBlob, num_entries : u8, schema : u16) -> io::Result<Vec<(u16, u16, u16, u16, u32)>>
    {
        // Language file V2 uses 32 bit offsets
        let mut tmp_info = Vec::new();
        let mut hits = HashSet::new();
  
        for _i in 0..num_entries {
            let mut buf = [0; 8];
            fp.read_exact(& mut buf) ?;
            let product_id = little_endian_2_bytes(&buf[2..4]);
            let derivative_id_low = buf[1] as u16; 
            let derivative_id_high = buf[1] as u16; 
            if !hits.insert((product_id, derivative_id_low)) {
                panic!("Duplicate product found!");
            }
            let flags = buf[0] as u16;
            if flags > 15 {
                panic!("Invalid flags in product index")
            }
            let offset = little_endian_4_bytes(&buf[4..8]);
            tmp_info.push((product_id, derivative_id_low, derivative_id_high, flags, offset))
        }
        return Result::Ok(tmp_info);
    }


    fn read_v3_entries(fp : & mut FileBlob, num_entries : u8, schema : u16) -> io::Result<Vec<(u16, u16, u16, u16, u32)>>
    {
        // Language file >= V3 uses 24 bit offsets
        let mut tmp_info = Vec::new();
//        let mut hits = HashSet::new();
  
        for _i in 0..num_entries {
            let mut buf = [0; 11];
            fp.read_exact(& mut buf) ?;
            let product_id = little_endian_2_bytes(&buf[0..2]);
            let derivative_id_low = little_endian_2_bytes(&buf[2..4]);
            let derivative_id_high = little_endian_2_bytes(&buf[4..6]);
            let flags = little_endian_2_bytes(&buf[6..8]);
            let offset = little_endian_3_bytes(&buf[8..11]);
            tmp_info.push((product_id, derivative_id_low, derivative_id_high, flags, offset))
        }
        return Result::Ok(tmp_info);
    }

    fn display(&self)
    {
        println!("Num of Product = {}", self.products.len());
    }
}


impl ProductIndexEntry {

    fn display(&self)
    {
//        let num_modes = self.mode_index.modes.len();
//        if self.derivative_id_high == 65535 &&  self.derivative_id_low == 0{
//            println!("Product = {} : ALL : num of modes = {}", self.product_id, num_modes);
//        } else if self.derivative_id_high > self.derivative_id_low {
//            println!("Product = {} : {} - {} : num_of_modes = {}", self.product_id, 
//                    self.derivative_id_low, self.derivative_id_high, num_modes);
//        } else {
//            println!("Product = {} : {} : num_of_modes = {}", self.product_id, self.derivative_id_low, num_modes);
//        }
    }
}

