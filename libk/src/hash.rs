pub fn hash_128bit(data: &[u8]) -> [u8; 16] {

    let mut salt0: u32 = 0x6a09e667;
    let mut salt1: u32 = 0xbb67ae85;
    let mut salt2: u32 = 0x3c6ef372;
    let mut salt3: u32 = 0xa54ff53a;

    for &byte in data {
        salt0 = salt0.wrapping_mul(31).wrapping_add(byte as u32);
        salt1 = salt1.wrapping_mul(37).wrapping_add(byte as u32);
        salt2 = salt2.wrapping_mul(41).wrapping_add(byte as u32);
        salt3 = salt3.wrapping_mul(43).wrapping_add(byte as u32);

        salt0 = salt0.rotate_left(5) ^ salt1;
        salt1 = salt1.rotate_left(7) ^ salt2;
        salt2 = salt2.rotate_left(9) ^ salt3;
        salt3 = salt3.rotate_left(11) ^ salt0;
    }

    salt0 = salt0.wrapping_add(salt3.rotate_left(3));
    salt1 = salt1.wrapping_add(salt0.rotate_left(7));
    salt2 = salt2.wrapping_add(salt1.rotate_left(13));
    salt3 = salt3.wrapping_add(salt2.rotate_left(19));

    let mut result = [0u8; 16];

    result[0..4].copy_from_slice(&salt0.to_le_bytes());
    result[4..8].copy_from_slice(&salt1.to_le_bytes());
    result[8..12].copy_from_slice(&salt2.to_le_bytes());
    result[12..16].copy_from_slice(&salt3.to_le_bytes());

    result
}

pub fn hash_to_hex(hash: &[u8; 16]) -> [u8; 32] {
    let mut hex = [0u8; 32];
    let charset = b"0123456789abcdef";

    for i in 0..16 {
        hex[i*2] = charset[(hash[i] >> 4) as usize];
        hex[i*2 + 1] = charset[(hash[i] & 0xf) as usize];
    }

    hex
}
