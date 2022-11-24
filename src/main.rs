extern crate xml;

pub mod blob;
pub mod characters;
pub mod conversion;
pub mod fonts;
pub mod keypadstrs;
pub mod language;
pub mod menus;
pub mod enumerations;
pub mod modes;
pub mod parameters;
pub mod products;
pub mod units;

use std::fs;
fn main() {
    let _font_index = fonts::read_font_file("fonts.bft");
    let character_maps = characters::read_character_file("CharacterMaps.xml");

    let paths = fs::read_dir("./").unwrap();

    for path in paths {
        let os_filename = path.unwrap().file_name();
        let filename = os_filename.into_string().unwrap();
        if filename.ends_with(".bin") {
            let lang_v2 = language::read_language_file(&filename, character_maps.clone());
            lang_v2.write_text_file(&(filename + ".txt"));
        }
        //        println!("Name {}", filename);
    }
}

//  	  let _lang_v2 = language::read_language_file("languageV2.bin", character_maps.clone());
//    let _lang_v3 = language::read_language_file("languageV3.bin", character_maps);
