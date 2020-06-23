use std::convert::TryInto;

use aes::Aes256;
use block_modes::block_padding::NoPadding;
use block_modes::{BlockMode, Cbc, Ecb};

type Aes256Cbc = Cbc<Aes256, NoPadding>;
type Aes256Ecb = Ecb<aes::Aes256, NoPadding>;

fn concat(a: u64, b: u64) -> u128 {
    ((a as u128) << 64) | (b as u128)
}

fn unwrap_key(mut ciphertext: Vec<u64>, kek: &[u8], iv: &[u8]) -> Vec<u64> {
    let n = (ciphertext.len() - 1) as u64;

    let mut a = ciphertext[0];

    for j in (0..=5).rev() {
        for i in (1..=n).rev() {
            let t = n * j + i;

            let b = u128::from_be_bytes(
                Aes256Ecb::new_var(kek, iv)
                    .expect("could not create AES")
                    .decrypt_vec(&concat(a ^ t, ciphertext[i as usize]).to_be_bytes())
                    .expect("no decrypt")
                    .as_slice()
                    .try_into()
                    .unwrap(),
            );

            a = (b >> 64) as u64;
            ciphertext[i as usize] = (b & ((1 << 64) - 1)) as u64;
        }
    }

    assert_eq!(a, u64::from_be_bytes(iv.try_into().unwrap()));

    ciphertext.into_iter().skip(1).collect()
}

pub fn solve(payload: Vec<u8>) -> Vec<u8> {
    let (header, payload) = payload.split_at(32 + 8 + 40 + 16);
    let kek = &header[0..32];
    let kek_iv = &header[32..32 + 8];
    let encrypted_key = &header[32 + 8..32 + 8 + 40];
    let payload_iv = &header[32 + 8 + 40..32 + 8 + 40 + 16];

    let key = encrypted_key
        .chunks(std::mem::size_of::<u64>())
        .map(|chunk| u64::from_be_bytes(chunk.try_into().unwrap()))
        .collect::<Vec<_>>();

    let key = unwrap_key(key, kek, kek_iv);

    let mut blarg_key = Vec::new();
    for b in &key {
        blarg_key.extend(&b.to_be_bytes())
    }

    let aes = Aes256Cbc::new_var(&blarg_key, payload_iv).unwrap();
    aes.decrypt_vec(payload).unwrap()
}
