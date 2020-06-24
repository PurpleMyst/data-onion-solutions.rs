use std::convert::TryInto;
use std::mem::size_of;

use aes::Aes256;
use block_modes::block_padding::NoPadding;
use block_modes::{BlockMode, Cbc, Ecb};

type Aes256Cbc = Cbc<Aes256, NoPadding>;
type Aes256Ecb = Ecb<aes::Aes256, NoPadding>;

fn concat(a: u64, b: u64) -> u128 {
    ((a as u128) << 64) | (b as u128)
}

fn unwrap_key(ciphertext: &mut Vec<u64>, kek: &[u8], iv: &[u8]) {
    // n is the amount of 64-bit blocks of the plaintext
    let n = (ciphertext.len() - 1) as u64;

    let mut a = ciphertext[0];

    for j in (0..=5).rev() {
        for (i, c) in ciphertext.iter_mut().enumerate().skip(1).rev() {
            let t = n * j + i as u64;
            let mut block = concat(a ^ t, *c).to_be_bytes();

            // decrypt this block in place
            Aes256Ecb::new_var(kek, iv)
                .expect("could not create AES")
                .decrypt(&mut block)
                .expect("could not decrypt block");

            // separate it into its constituents
            let b = u128::from_be_bytes(block[..].try_into().unwrap());
            a = (b >> 64) as u64;
            *c = (b & ((1 << 64) - 1)) as u64;
        }
    }

    // if in debug mode, verify the IV
    debug_assert_eq!(a, u64::from_be_bytes(iv.try_into().unwrap()));

    // remove the extra block that shouldn't be part of the plaintext
    ciphertext.remove(0);
}

pub fn solve(payload: Vec<u8>) -> Vec<u8> {
    let (header, payload) = payload.split_at(32 + 8 + 40 + 16);
    let kek = &header[0..32];
    let kek_iv = &header[32..32 + 8];
    let key = &header[32 + 8..32 + 8 + 40];
    let payload_iv = &header[32 + 8 + 40..32 + 8 + 40 + 16];

    // Parse they key into 64-bit blocks
    let mut key = key
        .chunks(size_of::<u64>())
        .map(|chunk| u64::from_be_bytes(chunk.try_into().unwrap()))
        .collect::<Vec<_>>();

    // Apply the AES key unwrapping algorithm
    unwrap_key(&mut key, kek, kek_iv);

    // Then parse it back into bytes
    let key = key.into_iter().fold(Vec::new(), |mut v, n| {
        v.extend(&n.to_be_bytes()[..]);
        v
    });

    // Now us the unwrapped key to decrypt the payload
    let aes = Aes256Cbc::new_var(&key, payload_iv).unwrap();
    aes.decrypt_vec(payload).unwrap()
}
