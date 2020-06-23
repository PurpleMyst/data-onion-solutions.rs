use std::iter::once;

// TODO: refactor this

#[derive(Debug, Copy, Clone)]
struct IPHeader {
    source_address: u32,
    destination_address: u32,
    packet_len: usize,

    calculated_checksum: u16,
    expected_checksum: u16,
}

#[derive(Debug, Copy, Clone)]
struct UDPHeader {
    source_port: u16,
    destination_port: u16,
    datagram_len: u16,
    expected_checksum: u16,
}

/// Calculate the "16-bit one's complement of the one's complement sum" of the input.. whatever that is
fn calculate_checksum<I: IntoIterator<Item = u16>>(it: I) -> u16 {
    let mut checksum: u32 = it.into_iter().map(u32::from).sum();
    checksum = ((checksum & 0xFFFF0000) >> 16) + (checksum & 0x0000FFFF);
    checksum = ((checksum & 0xFFFF0000) >> 16) + (checksum & 0x0000FFFF);
    debug_assert!(checksum <= 0xFFFF);
    !(checksum as u16)
}

macro_rules! take {
    ($ty:ty => $expr:expr) => {{
        use std::convert::TryInto;
        <$ty>::from_be_bytes($expr[..::std::mem::size_of::<$ty>()].try_into().unwrap())
    }};
}

impl IPHeader {
    fn parse(header: &[u8]) -> IPHeader {
        debug_assert_eq!(header.len(), 20);
        debug_assert_eq!(header[0], 0x45);

        IPHeader {
            source_address: take!(u32 => header[12..]),
            destination_address: take!(u32 => header[16..]),
            packet_len: usize::from(take!(u16 => header[2..])),
            calculated_checksum: calculate_checksum(
                (0..20)
                    .step_by(2)
                    .filter(|&i| i != 10)
                    .map(|i| take!(u16 => header[i..])),
            ),
            expected_checksum: take!(u16 => header[10..]),
        }
    }
}

impl UDPHeader {
    fn parse(header: &[u8]) -> UDPHeader {
        debug_assert_eq!(header.len(), 8);

        UDPHeader {
            source_port: take!(u16 => header[0..]),
            destination_port: take!(u16 => header[2..]),
            datagram_len: take!(u16 => header[4..]),
            expected_checksum: take!(u16 => header[6..]),
        }
    }
}

fn split_u32(n: u32) -> impl Iterator<Item = u16> {
    once(((n & 0xFFFF0000) >> 16) as u16).chain(once(((n & 0x0000FFFF) >> 0) as u16))
}

fn should_include(ip_header: IPHeader, udp_header: UDPHeader, datagram: &[u8]) -> bool {
    ip_header.calculated_checksum == ip_header.expected_checksum
        && ip_header.source_address == 0x0a_01_01_0a
        && ip_header.destination_address == 0x0a_01_01_c8
        && udp_header.destination_port == 420_69
        && calculate_checksum(
            split_u32(ip_header.source_address)
                .chain(split_u32(ip_header.destination_address))
                .chain(once(0x0011))
                .chain(once(udp_header.datagram_len))
                .chain(once(udp_header.source_port))
                .chain(once(udp_header.destination_port))
                .chain(once(udp_header.datagram_len))
                .chain((8..datagram.len()).step_by(2).map(|i| {
                    u16::from_be_bytes([datagram[i], *datagram.get(i + 1).unwrap_or(&0)])
                })),
        ) == udp_header.expected_checksum
}

pub(super) fn solve(payload: Vec<u8>) -> Vec<u8> {
    let mut offset = 0;
    let mut new_payload = Vec::with_capacity(payload.len());
    while offset < payload.len() {
        let ip_header = IPHeader::parse(&payload[offset..offset + 20]);
        let datagram = &payload[offset + 20..offset + ip_header.packet_len];
        let udp_header = UDPHeader::parse(&datagram[..8]);

        if should_include(ip_header, udp_header, datagram) {
            new_payload.extend(&datagram[8..]);
        }

        offset += ip_header.packet_len;
    }
    debug_assert_eq!(offset, payload.len());

    new_payload
}
