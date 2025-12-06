use std::fs;
use std::path::PathBuf;
use std::str;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

use data_onion::{layer1, layer2, layer3, layer4, layer5, layer6};

fn decode_payload(data: &[u8]) -> Vec<u8> {
    let text = str::from_utf8(data).expect("Payload is not valid UTF-8");
    let start_idx = text.rfind("<~").expect("Could not find Ascii85 start marker '<~'");
    ascii85::decode(&text[start_idx..]).expect("Failed to decode Ascii85")
}

fn bench_layers(c: &mut Criterion) {
    let mut group = c.benchmark_group("Onion Layers");

    let payload_0_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("payloads")
        .join("0.txt");
    
    let raw_0 = fs::read(&payload_0_path).expect("Failed to read payloads/0.txt");
    let wrapped_l1 = decode_payload(&raw_0); 

    let input_l1 = decode_payload(&wrapped_l1);

    let wrapped_l2 = layer1::solve(input_l1.clone());
    let input_l2 = decode_payload(&wrapped_l2);

    let wrapped_l3 = layer2::solve(input_l2.clone());
    let input_l3 = decode_payload(&wrapped_l3);

    let wrapped_l4 = layer3::solve(input_l3.clone());
    let input_l4 = decode_payload(&wrapped_l4);

    let wrapped_l5 = layer4::solve(input_l4.clone());
    let input_l5 = decode_payload(&wrapped_l5);

    let wrapped_l6 = layer5::solve(input_l5.clone());
    let input_l6 = decode_payload(&wrapped_l6);

    group.bench_function("Layer 1", |b| {
        b.iter_batched(
            || input_l1.clone(),
            |data| layer1::solve(data),
            BatchSize::SmallInput
        )
    });

    group.bench_function("Layer 2", |b| {
        b.iter_batched(
            || input_l2.clone(),
            |data| layer2::solve(data),
            BatchSize::SmallInput
        )
    });

    group.bench_function("Layer 3", |b| {
        b.iter_batched(
            || input_l3.clone(),
            |data| layer3::solve(data),
            BatchSize::LargeInput
        )
    });

    group.bench_function("Layer 4", |b| {
        b.iter_batched(
            || input_l4.clone(),
            |data| layer4::solve(data),
            BatchSize::LargeInput
        )
    });

    group.bench_function("Layer 5", |b| {
        b.iter_batched(
            || input_l5.clone(),
            |data| layer5::solve(data),
            BatchSize::LargeInput
        )
    });

    group.bench_function("Layer 6", |b| {
        b.iter_batched(
            || input_l6.clone(),
            |data| layer6::solve(data),
            BatchSize::LargeInput
        )
    });

    group.finish();
}

criterion_group!(benches, bench_layers);
criterion_main!(benches);
