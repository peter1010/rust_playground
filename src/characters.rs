use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::rc::Rc;
use std::vec::Vec;
use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};

#[derive(Clone)]
pub struct CharacterMaps {
    is_utf8: bool,
    maps: Rc<_CharacterMaps>,
}

struct _CharacterMaps {
    maps: Vec<CharacterMap>,
}

pub struct CharacterMap {
    id: u16,
    bytes_per: u16,
    chars: HashMap<u16, Character>,
}

struct Character {
    unicode: String,
    count: u32,
}

impl _CharacterMaps {
    fn empty() -> _CharacterMaps {
        _CharacterMaps {
            maps: Vec::<CharacterMap>::new(),
        }
    }

    fn new(maps: Vec<CharacterMap>) -> _CharacterMaps {
        _CharacterMaps { maps }
    }
}

impl CharacterMaps {
    pub fn is_utf8(&self) -> bool {
        self.is_utf8
    }

    pub fn utf8() -> CharacterMaps {
        CharacterMaps {
            is_utf8: true,
            maps: Rc::new(_CharacterMaps::empty()),
        }
    }

    pub fn decode_2bytes(&self, ch: u16) -> Option<String> {
        for map in &self.maps.maps {
            if map.bytes_per == 2 {
                let unicode = map.get_unicode(ch);
                return Some(unicode);
            }
        }
        panic!("Failed to decode 2 byte code {}", ch);
    }

    pub fn decode_byte(&self, ch: u8) -> Option<String> {
        for map in &self.maps.maps {
            if map.bytes_per == 1 {
                let unicode = map.get_unicode(ch as u16);
                return Some(unicode);
            }
        }
        panic!("Failed to decode 1 byte code {}", ch);
    }
}

impl PartialEq for CharacterMaps {
    fn eq(&self, other: &CharacterMaps) -> bool {
        self == other
    }
}

impl CharacterMap {
    fn new(attributes: &Vec<OwnedAttribute>) -> CharacterMap {
        let mut id = 0;
        let mut bytes_per = 0;
        for attr in attributes {
            match attr.name.local_name.as_str() {
                "id" => id = attr.value.parse().unwrap(),
                "bytesPerCharacter" => bytes_per = attr.value.parse().unwrap(),
                _ => {}
            };
        }
        CharacterMap {
            id: id,
            bytes_per,
            chars: HashMap::<u16, Character>::new(),
        }
    }

    fn get_unicode(&self, ch: u16) -> String {
        match self.chars.get(&ch) {
            Some(ch) => ch,
            None => {
                self.display();
                panic!(
                    "Failed to find {} in character map {} size {}",
                    ch, self.id, self.bytes_per
                )
            }
        }
        .get_unicode()
    }

    fn display(&self) {
        println!(
            "Character Map {}, size of chars {}",
            self.id, self.bytes_per
        );
        for (value, ch) in &self.chars {
            ch.display(*value);
        }
    }
}

impl Character {
    fn new(unicode: String) -> Character {
        Character {
            unicode: unicode,
            count: 0,
        }
    }

    fn get_unicode(&self) -> String {
        //        self.count += 1;
        self.unicode.clone()
    }

    fn display(&self, value: u16) {
        println!("{} => {} / count = {}", value, self.unicode, self.count);
    }

    fn create_from_xml(attributes: &Vec<OwnedAttribute>) -> (u16, Character) {
        let mut unicode: String = "".to_string();
        let mut value = 0;
        for attr in attributes {
            match attr.name.local_name.as_str() {
                "name" => unicode = attr.value.clone(),
                "value" => value = attr.value.parse().unwrap(),
                _ => {}
            };
        }
        (value, Character::new(unicode))
    }
}

/// Some XML starts with a BOM that causes issues!
fn skip_bom(fp: &mut BufReader<File>) {
    let mut bom = [0; 4];
    match fp.read_exact(&mut bom) {
        Ok(num) => num,
        Err(_) => {
            panic!("Failed to read XML BOM");
        }
    }
    if bom[0] == 0xEF {
        fp.seek_relative(-1).unwrap();
    } else {
        fp.seek_relative(-4).unwrap();
    }
}

pub fn read_character_file(filepath: &str) -> CharacterMaps {
    let fp = match File::open(filepath) {
        Ok(fp) => fp,
        Err(_) => {
            panic!("Failed to open {}", String::from(filepath));
        }
    };
    let mut fp = BufReader::new(fp);

    skip_bom(&mut fp);

    let parser = EventReader::new(fp);

    let mut maps = Vec::new();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            }) => {
                match name.local_name.as_str() {
                    "characterMap" => {
                        maps.push(CharacterMap::new(&attributes));
                    }
                    "char" => {
                        let (value, char_def) = Character::create_from_xml(&attributes);
                        let end = maps.len() - 1;
                        // println!("{} = {}", value, unicode);
                        maps[end].chars.insert(value, char_def);
                    }
                    _ => {}
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
            _ => {}
        }
    }
    return CharacterMaps {
        is_utf8: false,
        maps: Rc::new(_CharacterMaps::new(maps)),
    };
}
