const KNOWN_START: &[u8] = &*b"==[ Layer 4/5: ";
const KEY_LEN: usize = 32;

pub fn solve(mut payload: Vec<u8>) -> Vec<u8> {
    let mut known_key = [0u8; KEY_LEN];

    // We know what the start's supposed to be, so we can already calculate that bit
    KNOWN_START
        .iter()
        .zip(payload.iter())
        .zip(known_key.iter_mut())
        .for_each(|((a, b), out)| *out = a ^ b);

    // Now let's calculate the other indices
    let first_unknown_idx = known_key.iter().position(|&n| n == 0).unwrap();
    for (idx, key_byte) in known_key.iter_mut().enumerate().skip(first_unknown_idx) {
        // Round up all the bytes which this key byte would affect
        let chunks = payload
            .iter()
            .copied()
            .skip(idx)
            .step_by(KEY_LEN)
            .collect::<Vec<u8>>();

        // Find the byte which would cause only valid characters to be in the output
        // The payload is specially crafted such that there is only one such byte
        *key_byte = (0..u8::MAX)
            .find(|&guess| {
                !chunks
                    .iter()
                    .map(|c| c ^ guess)
                    .any(|c| !c.is_ascii() || (c < b' ' && c != b'\n'))
            })
            .unwrap();
    }

    // Actually apply the key now
    payload
        .iter_mut()
        .zip(known_key.iter().copied().cycle())
        .for_each(|(a, b)| *a ^= b);

    payload
}
