use crate::conversion::{little_endian_2_bytes, little_endian_4_bytes};
use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};
use std::vec::Vec;

pub struct FontIndex {
    sections: Vec<FontSection>,
}

struct FontSection {
    char_map: u8,
    font_family: u8,
    min_codepoint: u16,
    max_codepoint: u16,
    glyph_width: u8,
    glyph_height: u8,
    bytes_per_glyph: u8,
    blob: Vec<u8>,
}

impl FontIndex {
    pub fn from(fp: &mut File) -> io::Result<FontIndex> {
        // read font file header..
        let mut file_header = [0; 16];
        fp.read_exact(&mut file_header)?;

        let file_len = little_endian_4_bytes(&file_header[0..4]);
        let file_crc = little_endian_4_bytes(&file_header[4..8]);
        let schema = little_endian_2_bytes(&file_header[8..10]);
        let font_version = little_endian_2_bytes(&file_header[10..12]);
        let num_fonts = little_endian_2_bytes(&file_header[12..14]);
        let offset_to_offset_table = little_endian_2_bytes(&file_header[14..16]);

        println!("Font file length = {}, crc = {}", file_len, file_crc);
        println!("Font file schema {}, version {}", schema, font_version);
        println!("Number of fonts is {}", num_fonts);

        // Read the offset table..
        let mut offset_table = Vec::<u32>::new();
        fp.seek(SeekFrom::Start(offset_to_offset_table as u64))?;
        for _i in 0..num_fonts {
            let mut buf = [0; 4];
            fp.read_exact(&mut buf)?;
            offset_table.push(little_endian_4_bytes(&buf));
        }

        let mut sections = Vec::new();

        for i in 0..num_fonts {
            fp.seek(SeekFrom::Start(offset_table[i as usize] as u64))?;
            sections.push(FontSection::from(fp)?);
        }
        Result::Ok(FontIndex { sections })
    }

    pub fn get_size(&self, char_map: u8, font_family: u8) -> Option<(u8, u8)> {
        for section in self.sections.iter() {
            if (section.char_map == char_map) && (section.font_family == font_family) {
                return Some((section.glyph_width, section.glyph_height));
            }
        }
        return None;
    }

    pub fn get_glyph(&self, char_map: u8, font_family: u8, codepoint: u16) -> Option<Vec<u8>> {
        for section in self.sections.iter() {
            if (section.char_map == char_map)
                && (section.font_family == font_family)
                && (codepoint >= section.min_codepoint)
                && (codepoint <= section.max_codepoint)
            {
                let idx: usize = ((codepoint - section.min_codepoint) as usize)
                    * (section.bytes_per_glyph as usize);
                let mut glyph = Vec::<u8>::new();
                glyph.extend_from_slice(
                    &section.blob[idx..(idx + (section.bytes_per_glyph) as usize)],
                );
                return Some(glyph);
            }
        }
        None
    }
}

impl FontSection {
    pub fn from(fp: &mut File) -> io::Result<FontSection> {
        let mut font_header = [0; 12];
        fp.read_exact(&mut font_header)?;
        let char_map = font_header[0];
        let font_family = font_header[4];
        let glyph_width = font_header[5];
        let glyph_height = font_header[6];
        let bytes_per_glyph = font_header[7];
        let min_codepoint = little_endian_2_bytes(&font_header[8..10]);
        let max_codepoint = little_endian_2_bytes(&font_header[10..12]);
        println!(
            "map ={}, id = {}, {} x {}, {} to {}",
            char_map, font_family, glyph_width, glyph_height, min_codepoint, max_codepoint
        );
        let mut blob_size: usize =
            (bytes_per_glyph as usize) * ((max_codepoint - min_codepoint + 1) as usize);
        let mut buf = [0; 512];
        let mut blob = Vec::<u8>::new();
        while blob_size > 0 {
            match fp.read(&mut buf) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }
                    if bytes_read > blob_size {
                        blob.extend_from_slice(&buf[..blob_size]);
                        blob_size = 0;
                    } else {
                        blob.extend_from_slice(&buf[..bytes_read]);
                        blob_size -= bytes_read;
                    }
                }
                Err(_) => return Err(Error::from(ErrorKind::UnexpectedEof)),
            };
        }
        Result::Ok(FontSection {
            char_map,
            font_family,
            min_codepoint,
            max_codepoint,
            glyph_width,
            glyph_height,
            bytes_per_glyph,
            blob,
        })
    }
}

pub fn read_font_file(filepath: &str) -> FontIndex {
    let mut fp = match File::open(filepath) {
        Ok(fp) => fp,
        Err(_) => {
            panic!("Failed to open {}", String::from(filepath));
        }
    };

    let index = match FontIndex::from(&mut fp) {
        Ok(index) => index,
        Err(_) => {
            panic!("Failed to process {}", String::from(filepath));
        }
    };
    //    fp.close();
    return index;
}
