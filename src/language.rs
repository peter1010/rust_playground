use std::fs::File;
use std::io;
use std::io::Read;

use crate::conversion::{
    little_endian_2_bytes, little_endian_2_bytes_as_u8, little_endian_3_bytes,
    little_endian_4_bytes, little_endian_4_version,
};

use crate::blob::{FileBlob, BlobRegions};
use crate::characters::CharacterMaps;
use crate::keypadstrs::KeypadStrIndex;
//use crate::mnemonics::MnemonicIndex;
use crate::products::ProductIndex;
use crate::units::UnitsIndex;
use crate::enumerations::EnumerationsIndex;

pub struct Language {
    //    lang_name : [u8; 16],
    product_index: ProductIndex,
    enumeration_index: EnumerationsIndex,
    keypad_str_index: KeypadStrIndex,
    units_index: UnitsIndex,
}

impl Language {
    pub fn from(fp: &mut File, maps: CharacterMaps) -> io::Result<Language> {
        let mut common_hdr = [0; 32];
        fp.read_exact(&mut common_hdr)?;

        // Language file header
        let file_len = little_endian_4_bytes(&common_hdr[0..4]);
        let file_crc = little_endian_4_bytes(&common_hdr[4..8]);
        let schema = little_endian_2_bytes(&common_hdr[8..10]);
        let locale_id = little_endian_2_bytes(&common_hdr[10..12]);
        let lang_version = little_endian_4_version(&common_hdr[12..16]);
        let lang_name = &common_hdr[16..32];
        
        let mut fp = FileBlob::load(
            fp,
            file_len,
            file_crc,
            if schema > 3 {
                CharacterMaps::utf8()
            } else {
                maps
            },
        )?;
        fp.set_pos(32);
       
        println!("Language file locale_id {}, length {}, crc {}, schema {}", locale_id, file_len, file_crc, schema);

        let font_family = if schema < 4 {
            let mut font_hdr = [0; 2];
            fp.read_exact(&mut font_hdr, BlobRegions::Header);
            let font_family = little_endian_2_bytes_as_u8(&font_hdr[0..2]);
            println!("Font family {}", font_family);
            font_family
        } else {
            0
        };

        let mut hdr = [0; 2];
        fp.read_exact(&mut hdr, BlobRegions::Header);
        let offset_size = little_endian_2_bytes(&hdr[0..2]);

        println!(
            "Language file offset_size {}, version {}",
            offset_size, lang_version
        );

        Self::validate_schema(schema, offset_size);

        // Language file V2 uses 32 bit offsets, Language file >= V3 uses 24 bit offsets
        let offsets = Self::parse_offsets(&mut fp, schema, offset_size);

        fp.set_pos(offsets[0]);
        let product_index = ProductIndex::create_from_file(&mut fp, schema, font_family);

        fp.set_pos(offsets[1]);
        let enumeration_index = EnumerationsIndex::from(&mut fp, schema, font_family);

        let keypad_str_index = if offsets[2] > 0 {
            fp.set_pos(offsets[2]);
            KeypadStrIndex::from(&mut fp, schema, font_family)
        } else if schema == 2 {
            panic!("Missing Keypad strings in V2 language file");
        } else {
            KeypadStrIndex::empty()
        };

        fp.set_pos(offsets[3]);
        let units_index = UnitsIndex::from(&mut fp, schema, font_family);

        let lang = Language {
            product_index,
            enumeration_index,
            keypad_str_index,
            units_index,
        };

        println!("Products ....");

        for (product, details) in &lang.product_index {
            match details.to_string() {
                Ok(x) => println!("{} => {}", product, x),
                Err(x) => panic!("{} => {}", product, x),
            };
            for (mode, details) in details.get_modes() {
                match details.to_string(mode) {
                    Ok(x) => println!("- {}", x),
                    Err(x) => panic!("- {}", x),
                };
                for (menu, details) in details.get_menus() {
                    match details.to_string() {
                        Ok(x) => println!("- - M.{} => {}", menu, x),
                        Err(x) => panic!("- - M.{} => {}", menu, x),
                    };
                    for (param, details) in details.get_params() {
                        match details.to_string() {
                            Ok(x) => println!("- - - P.{} => {}", param, x),
                            Err(x) => panic!("- - - P.{} => {}", param, x),
                        };
                    }
                }
            }
        }

        println!("Legacy Enumerations ....");

        for (enumeration, details) in &lang.enumeration_index {
            match details.to_string() {
                Ok(x) => println!("{} => {}", enumeration, x),
                Err(x) => panic!("{} => {}", enumeration, x),
            };
        }

        println!("Keypad strs ....");

        for (num, details) in &lang.keypad_str_index {
            match details.to_string() {
                Ok(x) => println!("{} => {}", num, x),
                Err(x) => panic!("{} => {}", num, x),
            };
        }

        println!("Units ....");

        for (unit, details) in &lang.units_index {
            match details.to_string() {
                Ok(x) => println!("{} => {}", unit, x),
                Err(x) => panic!("{} => {}", unit, x),
            };
        }

        fp.display_stats();

        return Result::Ok(lang);
    }

    ///
    /// Validate the schema
    ///
    fn validate_schema(schema: u16, offset_size: u16) {
        match schema {
            2 => {
                if offset_size != 4 {
                    panic!("Invalid format")
                }
            }
            3 => {
                if offset_size != 3 {
                    panic!("Invalid format")
                }
            }
            4 => {
                if offset_size != 3 {
                    panic!("Invalid format")
                }
            }
            _ => panic!("Invalid format {}", schema),
        };
    }


    fn parse_offsets(fp : & mut FileBlob, schema : u16, offset_size: u16) -> Vec<u32> {
        // Language file V2 uses 32 bit offsets, Language file >= V3 uses 24 bit offsets
        let mut offsets = Vec::new();
        match schema {
            2 => {
                let mut header = [0; 16];
                fp.read_exact(&mut header, BlobRegions::Header); 
                offsets.push(little_endian_3_bytes(&header[0..4]));
                offsets.push(little_endian_3_bytes(&header[4..8]));
                offsets.push(little_endian_3_bytes(&header[8..12]));
                offsets.push(little_endian_3_bytes(&header[12..16]));
            }
            3 => {
                let mut header = [0; 12];
                fp.read_exact(&mut header, BlobRegions::Header); 
                offsets.push(little_endian_3_bytes(&header[0..3]));
                offsets.push(little_endian_3_bytes(&header[3..6]));
                offsets.push(little_endian_3_bytes(&header[6..9]));
                offsets.push(little_endian_3_bytes(&header[9..12]));
            }
            4 => {
                let mut header = [0; 9];
                fp.read_exact(&mut header, BlobRegions::Header); 
                offsets.push(little_endian_3_bytes(&header[0..3]));
                offsets.push(little_endian_3_bytes(&header[3..6]));
                offsets.push(0);
                offsets.push(little_endian_3_bytes(&header[6..9]));
            }
            _ => panic!("Invalid format"),
        };
        return offsets;
    }

    pub fn write_text_file(&self, filepath: &str) {
        let mut fp = match File::create(filepath) {
            Ok(fp) => fp,
            Err(_) => {
                panic!("Failed to open {}", String::from(filepath));
            }
        };
    }
}

pub fn read_language_file(filepath: &str, maps: CharacterMaps) -> Language {
    let mut fp = match File::open(filepath) {
        Ok(fp) => fp,
        Err(_) => {
            panic!("Failed to open {}", String::from(filepath));
        }
    };

    let language = match Language::from(&mut fp, maps) {
        Ok(index) => index,
        Err(_) => {
            panic!("Failed to process {}", String::from(filepath));
        }
    };
    return language;
}
