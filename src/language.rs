use std::fs::File;
use std::io::Read;
use std::io;
use std::collections::{HashMap, HashSet};

use crate::conversion::{
    little_endian_2_bytes, 
    little_endian_2_bytes_as_u8, 
    little_endian_3_bytes, little_endian_4_bytes, little_endian_4_version};

use crate::blob::{FileBlob, RawBlob};
use crate::mnemonics::MnemonicIndex;
use crate::keypadstrs::KeypadStrIndex;
use crate::units::UnitsIndex;
use crate::parameters::ParameterIndex;
use crate::menus::MenuIndex;
use crate::characters::CharacterMaps;
use crate::modes::ModeIndex;
use crate::products::ProductIndex;

pub struct Language {
    font_family : u8,
//    lang_name : [u8; 16],
    product_index : ProductIndex,
    mnemonic_index : MnemonicIndex,
    keypad_str_index : KeypadStrIndex,
    units_index : UnitsIndex
}


impl Language {

    pub fn from(fp : & mut File, maps : CharacterMaps) -> io::Result<Language> 
    {
        let mut header = [0; 52];
        fp.read_exact(& mut header) ?;

        // Language file header
        let file_len = little_endian_4_bytes(&header[0..4]);
        let file_crc = little_endian_4_bytes(&header[4..8]);
        let schema = little_endian_2_bytes(&header[8..10]);
        let locale_id = little_endian_2_bytes(&header[10..12]);
        let lang_version = little_endian_4_version(&header[12..16]);
        let lang_name = &header[16..32];
        let font_family = little_endian_2_bytes_as_u8(&header[32..34]);
        let offset_size = little_endian_2_bytes(&header[34..36]);

        println!("Language file length = {}, crc = {}", file_len, file_crc);
        println!("Language file schema {}, offset_size {}, version {}", schema, offset_size, lang_version);
        println!("Language file locale_id {}, font family {}", locale_id, font_family);  // lang_name..

        Self::validate_schema(schema, offset_size);

        // Language file V2 uses 32 bit offsets, Language file >= V3 uses 24 bit offsets
        let offsets = Self::parse_offsets(&header[36..], offset_size);

        let mut fp = FileBlob::load(fp, file_len, file_crc, if schema > 3 {CharacterMaps::utf8()} else {maps}) ?;

        fp.set_pos(offsets[0]);
        let product_index = ProductIndex::from(& mut fp, schema, font_family) ?;
        
        fp.set_pos(offsets[1]);
        let mnemonic_index = MnemonicIndex::from(& mut fp, schema, font_family) ?;
       
        let keypad_str_index = if offsets[2] > 0 {
            fp.set_pos(offsets[2]);
            KeypadStrIndex::from(& mut fp, schema, font_family) ?
        } else if schema == 2 {
            panic!("Missing Keypad strings in V2 language file");
        } else {
            KeypadStrIndex::empty()
        };

        fp.set_pos(offsets[3]);
        let units_index = UnitsIndex::from(& mut fp, schema, font_family) ?;

        let lang = Language { font_family, product_index, mnemonic_index, keypad_str_index, units_index };


        lang.display();

        let mut min_off = file_len;
        let mut max_off = 0;

        let mut offsets = HashSet::<u32>::new();

//        for product in &lang.product_index.products {
//            for mode in &product.mode_index.modes {
//                for menu in &mode.menu_index.menus {
//                    offsets.insert(menu.caption_off);
//                    offsets.insert(menu.tooltip_off);
//                    for param in &menu.param_index.params {
//                        offsets.insert(param.caption_off);
//                        offsets.insert(param.tooltip_off);
//                    }
//                }
//            }
//        }
        for (mnemonic, details) in &lang.mnemonic_index {
            offsets.insert(details.get_caption_off());
            match details.get_string() {
                Ok(x) => println!("{} => {}", mnemonic, x),
                Err(x) => panic!("{} => {}", mnemonic, x),
            };
        }
//        for keypad_str in &lang.keypad_str_index.keypad_strs {
//            offsets.insert(keypad_str.caption_off);
//        }
        for (unit, details) in &lang.units_index {
            offsets.insert(details.get_caption_off());
            offsets.insert(details.get_tooltip_off());
            match details.get_string() {
                Ok(x) => println!("{} => {}", unit, x),
                Err(x) => panic!("{} => {}", unit, x),
            };
        }
        offsets.remove(&0);
        println!("There are {} strings", offsets.len());

        let mut min_off = file_len;
        let mut max_off = 0;
        for offset in offsets {
            if offset < min_off {
                min_off = offset;
            }
            if offset > max_off {
                max_off = offset;
            }
        }
//        fp.Seek(FromStart(max_off));
        println!("There are {} - {} ", min_off, max_off);
        return Result::Ok(lang);
    }

    fn validate_schema(schema : u16, offset_size : u16) {
        match schema {
            2 => if offset_size != 4 {panic!("Invalid format")},
            3 => if offset_size != 3 {panic!("Invalid format")}
            _ => panic!("Invalid format")
        };
    }


    fn parse_offsets(header : &[u8], offset_size : u16) -> Vec<u32> 
    {
        // Language file V2 uses 32 bit offsets, Language file >= V3 uses 24 bit offsets
        let mut offsets = Vec::new();
        match offset_size {
            3 => {
                offsets.push(little_endian_3_bytes(&header[0..3]));
                offsets.push(little_endian_3_bytes(&header[3..6]));
                offsets.push(little_endian_3_bytes(&header[6..9]));
                offsets.push(little_endian_3_bytes(&header[9..12]));
            },
            4 => {
                offsets.push(little_endian_3_bytes(&header[0..4]));
                offsets.push(little_endian_3_bytes(&header[4..8]));
                offsets.push(little_endian_3_bytes(&header[8..12]));
                offsets.push(little_endian_3_bytes(&header[12..16]));
            },
            _ => panic!("Invalid format")
        };
        return offsets;
    }

    fn display(&self) 
    {
//        let product_index = &self.product_index;
//        product_index.display();
//        for product in &product_index.products {
//            product.display();
//            for mode in &product.mode_index.modes {
//                mode.display();
//                for menu in &mode.menu_index.menus {
//                    menu.display();
//                    menu.param_index.display();
//                    for param in &menu.param_index.params {
//                    }
//                }
//            }
//        }
//        for mmenonic in &self.mmenonic_index.mmenonics {
//       }
//        for keypad_str in &self.keypad_str_index.keypad_strs {
//        }
//        for unit in &self.units_index.units {
//        }
   
    }
}


pub fn read_language_file(filepath : &str, maps : CharacterMaps) -> Language
{
    let mut fp = match File::open(filepath) {
        Ok(fp) => fp,
        Err(_) => {
            panic!("Failed to open {}", String::from(filepath));
        }
    };

    let language = match Language::from(& mut fp, maps) {
        Ok(index) => index,
        Err(_) => {
            panic!("Failed to process {}", String::from(filepath));
        }
    };
    return language;
}
