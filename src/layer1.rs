pub fn solve(mut payload: Vec<u8>) -> Vec<u8> {
    // 1) Flip every second bit
    payload.iter_mut().for_each(|byte| *byte ^= 0x55);

    fn rotate(prev: u8, cur: u8) -> u8 {
        ((prev & 1) << 7) | (cur >> 1)
    }

    // 2) Rotate to the right
    let mut prev = *payload.last().unwrap();
    for cur in &mut payload {
        let next_prev = *cur;
        *cur = rotate(prev, *cur);
        prev = next_prev;
    }

    payload
}
