pub fn crc16_m17(data: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x5935;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}
