use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::conversion::{little_endian_2_bytes, little_endian_3_bytes, little_endian_4_bytes};

use crate::blob::{FileBlob, BlobRegions};
use crate::modes::ModeIndex;

///
/// ProductIndex is a dictionary of Products
///
pub struct ProductIndex {
    products: HashMap<u16, ProductIndexEntry>,
}

///
/// ProductIndexEntry is a entry in the dictionary of Products
///
pub struct ProductIndexEntry {
    product_id : u16, // Product Id is also the Key in the Dictionary in ProductIndex
    derivative_id_low: u16,
    derivative_id_high: u16,
    flags: u16,
    mode_index: Rc<ModeIndex>,
}

pub struct ProductIndexIterator {
    items: Vec<(u16, ProductIndexEntry)>,
}

///
/// Product Index
///
impl ProductIndex {

    ///
    /// Create a ProductIndex from the FileBlob
    ///
    pub fn from(fp: &mut FileBlob, schema: u16, font_family: u8) -> ProductIndex {
        let mut header = [0; 2];
        fp.read_exact(&mut header, BlobRegions::Products);

        // Product index header
        let num_products = header[0];
        let idx_entry_len = header[1];


        Self::validate_schema(schema, idx_entry_len, num_products);

        let tmp_info = match schema {
            2 => Self::read_v2_entries(fp, num_products),
            3 => Self::read_v3_entries(fp, num_products),
            4 => Self::read_v3_entries(fp, num_products),
            _ => panic!("Invalid format"),
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
            let mode_index = ModeIndex::from(fp, schema, font_family);
            products.insert(
                product_id,
                ProductIndexEntry::new(product_id, derivative_id_low, derivative_id_high, flags, mode_index),
            );
        }

        ProductIndex { products }
    }

    ///
    /// Valid the Product_Index
    fn validate_schema(schema: u16, idx_entry_len: u8, num_of_products: u8) {
        match schema {
            2 => {
                if idx_entry_len != 8 {
                    panic!("ProductIndexEntry wrong size 8 != {}", idx_entry_len)
                }
            }
            3 => {
                if idx_entry_len != 11 {
                    panic!("ProductIndexEntry wrong size 11 != {}", idx_entry_len)
                }
            }
            4 => {
                if idx_entry_len != 11 {
                    panic!("ProductIndexEntry wrong size 11 != {}", idx_entry_len)
                }
            }
 
            _ => panic!("Invalid format"),
        };
        if num_of_products > 40 {
            panic!("Seems a lot of products!");
        }
    }

    ///
    /// Parse V2 Product Index Entries intinally into a list of tuples
    ///
    fn read_v2_entries(
        fp: &mut FileBlob,
        num_entries: u8,
    ) -> Vec<(u16, u16, u16, u16, u32)> {
        // Language file V2 uses 32 bit offsets
        let mut tmp_info = Vec::new();
        let mut hits = HashSet::new();

        for _i in 0..num_entries {
            let mut buf = [0; 8];
            fp.read_exact(&mut buf, BlobRegions::Products);
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
            tmp_info.push((
                product_id,
                derivative_id_low,
                derivative_id_high,
                flags,
                offset,
            ))
        }
        tmp_info
    }

    ///
    /// Parse V3 Product Index Entries intinally into a list of tuples
    ///
    fn read_v3_entries(
        fp: &mut FileBlob,
        num_entries: u8,
    ) -> Vec<(u16, u16, u16, u16, u32)> {
        // Language file >= V3 uses 24 bit offsets
        let mut tmp_info = Vec::new();
        //        let mut hits = HashSet::new();

        for _i in 0..num_entries {
            let mut buf = [0; 11];
            fp.read_exact(&mut buf, BlobRegions::Products);
            let product_id = little_endian_2_bytes(&buf[0..2]);
            let derivative_id_low = little_endian_2_bytes(&buf[2..4]);
            let derivative_id_high = little_endian_2_bytes(&buf[4..6]);
            let flags = little_endian_2_bytes(&buf[6..8]);
            let offset = little_endian_3_bytes(&buf[8..11]);
            tmp_info.push((
                product_id,
                derivative_id_low,
                derivative_id_high,
                flags,
                offset,
            ))
        }
        tmp_info
    }
}

impl IntoIterator for &ProductIndex {
    type Item = (u16, ProductIndexEntry);
    type IntoIter = ProductIndexIterator;

    fn into_iter(self) -> Self::IntoIter {
        let mut keys = Vec::new();
        for key in self.products.keys() {
            keys.push(*key)
        }
        keys.sort();
        keys.reverse();
        let mut items = Vec::new();
        for key in keys {
            items.push((key, self.products[&key].clone()));
        }
        ProductIndexIterator { items }
    }
}

impl ProductIndexEntry {
    fn new(
        product_id : u16,
        derivative_id_low: u16,
        derivative_id_high: u16,
        flags: u16,
        mode_index: ModeIndex,
    ) -> ProductIndexEntry {
        ProductIndexEntry {
            product_id,
            derivative_id_low,
            derivative_id_high,
            flags,
            mode_index: Rc::<ModeIndex>::new(mode_index),
        }
    }

    pub fn to_string(&self) -> Result<String, String> {
        let num_modes = self.mode_index.get_num_modes();
        if self.derivative_id_high == 65535 && self.derivative_id_low == 0 {
            return Result::Ok(format!("ALL DERIVATIVES : num of modes = {}", num_modes));
        }
        if self.derivative_id_high > self.derivative_id_low {
            return Result::Ok(format!(
                "Derv {} - {} : num_of_modes = {}",
                self.derivative_id_low, self.derivative_id_high, num_modes
            ));
        }
        return Result::Ok(format!(
            " Derv {} : num_of_modes = {}",
            self.derivative_id_low, num_modes
        ));
    }

    pub fn get_modes(&self) -> &ModeIndex {
        &self.mode_index
    }
}

impl Clone for ProductIndexEntry {
    fn clone(&self) -> ProductIndexEntry {
        ProductIndexEntry {
            product_id: self.product_id,
            derivative_id_low: self.derivative_id_low,
            derivative_id_high: self.derivative_id_high,
            flags: self.flags,
            mode_index: self.mode_index.clone(),
        }
    }
}

impl Iterator for ProductIndexIterator {
    type Item = (u16, ProductIndexEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}
