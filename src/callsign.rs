fn char_to_base40(c: char) -> u64 {
    match c {
        'A'..='Z' => (c as u64 - 'A' as u64) + 1,
        '0'..='9' => (c as u64 - '0' as u64) + 27,
        '-'       => 37,
        '/'       => 38,
        '.'       => 39,
        _         => 0,
    }
}

fn base40_to_char(val: u64) -> char {
    match val {
        1..=26  => (('A' as u8) + (val as u8) - 1) as char,
        27..=36 => (('0' as u8) + (val as u8) - 27) as char,
        37      => '-',
        38      => '/',
        39      => '.',
        _       => ' ',
    }
}

pub fn encode_callsign(callsign: &str) -> [u8; 6] {
    let mut val: u64 = 0;
    let mut pow: u64 = 1;

    let mut chars: Vec<char> = callsign.to_uppercase().chars().collect();
    if chars.len() > 9 {
        chars.truncate(9);
    }
    while chars.len() < 9 {
        chars.push(' ');
    }

    for &c in chars.iter() {
        val += char_to_base40(c) * pow;
        pow *= 40;
    }

    let mut out = [0u8; 6];
    for i in 0..6 {
        out[5 - i] = ((val >> (8 * i)) & 0xFF) as u8;
    }
    out
}

pub fn decode_callsign(data: &[u8; 6]) -> String {
    let mut val: u64 = 0;
    for b in data.iter() {
        val = (val << 8) | (*b as u64);
    }

    match val {
        0x000000000000 => return "INVALID".to_string(),
        0xFFFFFFFFFFFF => return "BROADCAST".to_string(),
        0xEE6B28000000..=0xFFFFFFFFFFFE => {
            return format!("RESERVED-{:012X}", val);
        }
        _ => {}
    }

    let mut chars = Vec::new();
    let mut temp = val;
    for _ in 0..9 {
        let digit = temp % 40;
        temp /= 40;
        chars.push(base40_to_char(digit));
    }

    chars.into_iter().collect::<String>()
}

pub fn base_callsign(callsign: &str) -> String {
    let trimmed = callsign.trim();
    trimmed
        .split(|c| c == ' ' || c == '-' || c == '/')
        .next()
        .unwrap_or("")
        .to_uppercase()
}
