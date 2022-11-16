extern crate xml;

pub mod fonts;
pub mod language;
pub mod conversion;
pub mod blob;
pub mod mnemonics;
pub mod keypadstrs;
pub mod units;
pub mod parameters;
pub mod menus;
pub mod characters;
pub mod modes;
pub mod products;


fn main() {
    let _font_index = fonts::read_font_file("fonts.bft");
    let character_maps = characters::read_character_file("CharacterMaps.xml");

    let _lang_v2 = language::read_language_file("languageV2.bin", character_maps.clone());
    let _lang_v3 = language::read_language_file("languageV3.bin", character_maps);

}
