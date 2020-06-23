use std::fs;

use anyhow::{Context, Result};

type Solver = fn(Vec<u8>) -> Vec<u8>;

const LAYER_SOLVERS: &[Solver] = &[solve_layer1, solve_layer2, solve_layer3, solve_layer4];

fn load_payload(num: usize) -> Result<Vec<u8>> {
    let contents = fs::read_to_string(format!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/payloads/{}.dat"),
        num
    ))?;

    decode_payload(&contents)
}

fn decode_payload(flavor: &str) -> Result<Vec<u8>> {
    let contents = &flavor[flavor
        .rfind("<~")
        .context("Could not find start of ASCII85 payload")?..];

    ascii85::decode(contents).map_err(|err| anyhow::Error::msg(format!("ascii85 error: {}", err)))
}

fn write_payload(num: usize, data: &[u8]) -> Result<()> {
    fs::write(
        format!(concat!(env!("CARGO_MANIFEST_DIR"), "/payloads/{}.dat"), num),
        data,
    )?;
    Ok(())
}

fn solve_layer1(mut payload: Vec<u8>) -> Vec<u8> {
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

fn solve_layer2(mut payload: Vec<u8>) -> Vec<u8> {
    fn parity(mut byte: u8) -> u8 {
        let mut result = 0;
        while byte != 0 {
            result ^= byte & 1;
            byte >>= 1;
        }
        result
    }

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

fn solve_layer3(mut payload: Vec<u8>) -> Vec<u8> {
    const KNOWN_START: &[u8] = &*b"==[ Layer 4/5: ";
    const KEY_LEN: usize = 32;

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

mod layer4;
use layer4::solve as solve_layer4;

fn main() -> Result<()> {
    let payload = LAYER_SOLVERS.into_iter().enumerate().try_fold(
        load_payload(0)?,
        |payload, (idx, solver)| -> Result<Vec<u8>> {
            write_payload(idx + 1, &payload)?;
            Ok(solver(decode_payload(std::str::from_utf8(&payload)?)?))
        },
    )?;

    write_payload(LAYER_SOLVERS.len() + 1, &payload)?;
    Ok(())
}
