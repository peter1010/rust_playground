
pub fn little_endian_4_bytes(bytes : &[u8]) -> u32 
{
    (bytes[0] as u32) | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16) | ((bytes[3] as u32) << 24)
}

pub fn little_endian_3_bytes(bytes : &[u8]) -> u32 
{
    (bytes[0] as u32) | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16)
}

pub fn little_endian_2_bytes(bytes : &[u8]) -> u16 
{
    (bytes[0] as u16) | ((bytes[1] as u16) << 8)
}

pub fn little_endian_2_bytes_as_u8(bytes : &[u8]) -> u8
{
    if bytes[1] != 0 {
        panic!("Too large");
    }
    bytes[0]
}


pub fn little_endian_4_version(bytes : &[u8]) -> String
{
    let major = bytes[3];
    let minor = bytes[2];
    let patch = bytes[1];
    let build = bytes[0];

    format!("V{}.{}.{}.{}", major, minor, patch, build)
}
