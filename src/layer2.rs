fn parity(byte: u8) -> u8 {
    ((byte >> 0) & 1)
        ^ ((byte >> 1) & 1)
        ^ ((byte >> 2) & 1)
        ^ ((byte >> 3) & 1)
        ^ ((byte >> 4) & 1)
        ^ ((byte >> 5) & 1)
        ^ ((byte >> 6) & 1)
        ^ ((byte >> 7) & 1)
}

pub fn solve(mut payload: Vec<u8>) -> Vec<u8> {
    // 1) Retain only bytes with the correct parity bit
    payload.retain(|&byte| parity(byte & !1) == byte & 1);

    // 2) For each group of 8 bytes, form 7 bytes
    let mut next_payload = Vec::with_capacity(payload.len() * 7 / 8);
    payload.chunks_exact(8).for_each(|chunk| {
        let decoded = chunk
            .into_iter()
            .fold(0u64, |acc, byte| (acc << 7) | u64::from(byte >> 1));

        next_payload.extend(&decoded.to_be_bytes()[1..])
    });
    next_payload
}
