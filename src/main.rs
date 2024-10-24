use std::fs;

// An ID3v2 tag can be detected with the following pattern:
//      $49 44 33 yy yy xx zz zz zz zz
//    Where yy is less than $FF, xx is the 'flags' byte and zz is less than
//    $80.

#[derive(Debug)]
struct Id3v2Header {
    identifier: [u8; 3],
    version: [u8; 2],
    flags: u8,
    size: u32,
}

#[derive(Debug)]
struct Id3v2ExtendedHeader {
    size: u32,
    number_of_flag_bytes: u8,
    flags: u8,
    b_flag_length: u8,
    c_flag_length: u8,
    total_frame_crc: Option<u64>,
    d_flag_length: u8,
    restrictions: Option<u8>,
}

#[derive(Debug)]
struct Id3v2Frame {
    identifier: String,
    size: u32,
    flags: [u8; 2],
}

fn convert_safesynch_to_u32(byte0: u8, byte1: u8, byte2: u8, byte3: u8) -> u32 {
    u32::from(byte0) << 21 | u32::from(byte1) << 14 | u32::from(byte2) << 7 | u32::from(byte3)
}

fn convert_safesynch_to_u64(byte0: u8, byte1: u8, byte2: u8, byte3: u8, byte4: u8) -> u64 {
    u64::from(byte0) << 28
        | u64::from(byte1) << 21
        | u64::from(byte2) << 14
        | u64::from(byte3) << 7
        | u64::from(byte4)
}

fn parse_extended_header(bytes: &Vec<u8>) -> Id3v2ExtendedHeader {
    let size: u32 = convert_safesynch_to_u32(bytes[0], bytes[1], bytes[2], bytes[3]);
    let number_of_flag_bytes = bytes[4];
    let flags = bytes[5];

    let b_flag_length = bytes[6];
    let c_flag_length = bytes[7];

    let mut total_frame_crc: Option<u64> = None;

    // CRC data flag is set
    if flags & 0b0010000 != 0 {
        total_frame_crc = Some(convert_safesynch_to_u64(
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12],
        ))
    }

    // positions change based on if c flag was set and its data followed the c flag length byte
    let d_flag_length = if flags & 0b0010000 != 0 {
        bytes[13]
    } else {
        bytes[8]
    };
    let mut restrictions: Option<u8> = if flags & 0b0010000 != 0 {
        Some(bytes[14])
    } else {
        Some(bytes[9])
    };

    restrictions = if flags & 0b00010000 != 0 {
        restrictions
    } else {
        None
    };

    println!("extended_header_size: {:#?}", size);
    // println!("extended_header_size_bytes: {:#04X?}", extended_header_size_bytes);
    println!(
        "extended_header_number_flag_bytes: {:#?}",
        number_of_flag_bytes
    );
    println!("extended_header_flags: {:#?}", flags);
    println!("b_flag_length: {:#?}", b_flag_length);
    println!("c_flag_length: {:#?}", c_flag_length);
    println!("total_frame_crc: {:#?}", total_frame_crc);
    println!("d_flag_length: {:#?}", d_flag_length);
    println!(
        "restrictions: {:#08b}",
        match restrictions {
            Some(x) => x,
            None => 0,
        }
    );

    Id3v2ExtendedHeader {
        size,
        number_of_flag_bytes,
        flags,
        b_flag_length,
        c_flag_length,
        total_frame_crc,
        d_flag_length,
        restrictions,
    }
}

fn parse_header(bytes: &Vec<u8>) -> Id3v2Header {
    let identifier = [bytes[0], bytes[1], bytes[2]];
    let version = [bytes[3], bytes[4]];
    let flags = bytes[5];
    let size: u32 = convert_safesynch_to_u32(bytes[6], bytes[7], bytes[8], bytes[9]);

    Id3v2Header {
        identifier,
        version,
        flags,
        size,
    }
}

fn extract_tag(bytes: &Vec<u8>) -> Vec<u8> {
    // add 10 to include header size
    let total_tag_size = convert_safesynch_to_u32(bytes[6], bytes[7], bytes[8], bytes[9]) + 10;
    let mut total_tag_size = usize::try_from(total_tag_size).unwrap();

    println!("total tag size: {:#?}", total_tag_size);

    // Footer might be present
    // Footer identifier is in reverse if file is read from start of file
    if bytes[total_tag_size + 10] == 0x33
        && bytes[total_tag_size + 11] == 0x44
        && bytes[total_tag_size + 12] == 0x49
    {
        total_tag_size += 10;
    }

    bytes[..total_tag_size].to_vec()
}

fn main() {
    let path = std::env::args().nth(1).expect("must give a path");
    let bytes = fs::read(path).expect("unable to read file");

    // parser does not work with tags that don't follow ID3v2.4 format
    if bytes[0] != 0x49 || bytes[1] != 0x44 || bytes[2] != 0x33 {
        // Not an ID3v2 tag
        println!("\n[Error] ID3v2 not present within file\n");
        return;
    }

    let id3v2_bytes = extract_tag(&bytes);

    for byte in &id3v2_bytes {
        print!("{:#04X?} ", byte);
    }

    let header_bytes = &id3v2_bytes[..10];
    let header = parse_header(&header_bytes.to_vec());
    let extended_header = if header.flags & 0b01000000 != 0 {
        let total_extended_header_size = convert_safesynch_to_u32(
            id3v2_bytes[11],
            id3v2_bytes[12],
            id3v2_bytes[13],
            id3v2_bytes[14],
        );

        let total_extended_header_size = usize::try_from(total_extended_header_size).unwrap();
        let extended_header_bytes = &id3v2_bytes[11..total_extended_header_size].to_vec();
        Some(parse_extended_header(extended_header_bytes))
    } else {
        None
    };

    let footer_present = id3v2_bytes[id3v2_bytes.len() - 9] == 0x33
        && id3v2_bytes[id3v2_bytes.len() - 10] == 0x44
        && id3v2_bytes[id3v2_bytes.len() - 11] == 0x49;

    let frames_start = if extended_header.is_some() {
        usize::try_from(header.size + extended_header.as_ref().unwrap().size + 10).unwrap()
    } else {
        usize::try_from(header.size + 10).unwrap()
    };

    let frames_end = if footer_present {
        id3v2_bytes.len() - 10
    } else {
        id3v2_bytes.len()
    };

    let frame_bytes = &id3v2_bytes[frames_start..frames_end];

    let footer_bytes = if footer_present {
        Some(&id3v2_bytes[frames_end + 1..])
    } else {
        None
    };

    println!("\nheader: {:#04X?}", header);
    println!("extended header: {:#04X?}", extended_header);
    println!("frame_bytes: {:#04X?}", frame_bytes);
    println!("footer_bytes: {:#04X?}", footer_bytes);
}
