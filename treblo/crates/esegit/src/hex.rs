const LOWER_HEX_CHARS: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

pub fn to_hex_string(bs: &[u8]) -> String {
    let mut s = String::with_capacity(bs.len() * 2);
    for b in bs.iter() {
        s.push(LOWER_HEX_CHARS[((b & 0b11110000) >> 4) as usize]);
        s.push(LOWER_HEX_CHARS[(b & 0b00001111) as usize]);
    }
    s
}
