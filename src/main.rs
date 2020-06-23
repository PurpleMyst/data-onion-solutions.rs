use std::fs;
use std::io;

type Solver = fn(Vec<u8>) -> Vec<u8>;

mod layer1;
mod layer2;
mod layer3;
mod layer4;
mod layer5;

const LAYER_SOLVERS: &[Solver] = &[
    layer1::solve,
    layer2::solve,
    layer3::solve,
    layer4::solve,
    layer5::solve,
];

fn load_payload(num: usize) -> io::Result<Vec<u8>> {
    fs::read_to_string(format!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/payloads/{}.txt"),
        num
    ))
    .map(|text| decode_payload(&text))
}

fn decode_payload(flavor: &str) -> Vec<u8> {
    let contents = &flavor[flavor.rfind("<~").unwrap()..];
    ascii85::decode(contents).unwrap()
}

fn write_payload(num: usize, data: &[u8]) -> io::Result<()> {
    fs::write(
        format!(concat!(env!("CARGO_MANIFEST_DIR"), "/payloads/{}.txt"), num),
        data,
    )
}

fn main() -> io::Result<()> {
    let payload = LAYER_SOLVERS.into_iter().enumerate().try_fold(
        load_payload(0)?,
        |payload, (idx, solver)| -> io::Result<Vec<u8>> {
            write_payload(idx + 1, &payload)?;
            let payload = std::str::from_utf8(&payload).unwrap();
            Ok(solver(decode_payload(payload)))
        },
    )?;

    write_payload(LAYER_SOLVERS.len() + 1, &payload)?;
    Ok(())
}
