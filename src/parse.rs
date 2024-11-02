use crate::extract;
use crate::tag;
use crate::utility;

pub fn parse_tag(bytes: &Vec<u8>) -> Result<tag::Id3v2Tag, String> {
    if bytes[0] != 0x49 || bytes[1] != 0x44 || bytes[2] != 0x33 {
        // Not an ID3v2 tag
        return Err(format!(
            "not a valid ID3v2.4 tag ({:#04X?} {:#04X?} {:#04X?})",
            bytes[0], bytes[1], bytes[2]
        ));
    }

    // for byte in bytes {
    //     print!("{:#04X?} ", byte);
    // }

    let header_bytes = &bytes[..10];
    let header = parse_header(&header_bytes.to_vec());
    let extended_header = if header.flags & 0b01000000 != 0 {
        let total_extended_header_size =
            utility::convert_safesynch_to_u32(bytes[11], bytes[12], bytes[13], bytes[14]);

        let total_extended_header_size = usize::try_from(total_extended_header_size).unwrap();
        let extended_header_bytes = &bytes[11..total_extended_header_size].to_vec();
        Some(parse_extended_header(extended_header_bytes))
    } else {
        None
    };

    let footer_present = bytes[bytes.len() - 9] == 0x33
        && bytes[bytes.len() - 10] == 0x44
        && bytes[bytes.len() - 11] == 0x49;

    // header is always 10 bytes
    // extended header might or might not be present
    // frames start after extended up to footer
    let frames_start = if extended_header.is_some() {
        usize::try_from(extended_header.as_ref().unwrap().size + 10).unwrap()
    } else {
        10
    };

    let frames_end = if footer_present {
        bytes.len() - 10
    } else {
        bytes.len()
    };

    let frame_bytes = &bytes[frames_start..frames_end].to_vec();
    let frames = parse_frames(frame_bytes);
    let footer: Option<tag::Id3v2Header> = if footer_present {
        Some(parse_header(&bytes[frames_end + 1..].to_vec()))
    } else {
        None
    };

    Ok(tag::Id3v2Tag {
        header,
        extended_header,
        frames,
        footer,
    })
}

fn parse_extended_header(bytes: &Vec<u8>) -> tag::Id3v2ExtendedHeader {
    let size: u32 = utility::convert_safesynch_to_u32(bytes[0], bytes[1], bytes[2], bytes[3]);
    let number_of_flag_bytes = bytes[4];
    let flags = bytes[5];

    let b_flag_length = bytes[6];
    let c_flag_length = bytes[7];

    let mut total_frame_crc: Option<u64> = None;

    // CRC data flag is set
    if flags & 0b0010000 != 0 {
        total_frame_crc = Some(utility::convert_safesynch_to_u64(
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

    tag::Id3v2ExtendedHeader {
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

fn parse_header(bytes: &Vec<u8>) -> tag::Id3v2Header {
    let identifier = [bytes[0], bytes[1], bytes[2]];
    let version = [bytes[3], bytes[4]];
    let flags = bytes[5];
    let size: u32 = utility::convert_safesynch_to_u32(bytes[6], bytes[7], bytes[8], bytes[9]);

    tag::Id3v2Header {
        identifier,
        version,
        flags,
        size,
    }
}

fn parse_frame(bytes: &Vec<u8>) -> Result<tag::Frame, String> {
    let identifier = [bytes[0], bytes[1], bytes[2], bytes[3]];
    let size = utility::convert_safesynch_to_u32(bytes[4], bytes[5], bytes[6], bytes[7]);
    let flags = [bytes[8], bytes[9]];

    let header = tag::Id3v2FrameHeader {
        identifier,
        size,
        flags,
    };

    let data = bytes[10..].to_vec();
    let binding = String::from_utf8(identifier.to_vec()).unwrap();
    let ascii_id = binding.as_str();

    match ascii_id {
        "TIT2" | "TALB" | "TPE1" | "TSSE" => Ok(tag::Frame::Text(tag::Id3v2TextFrame {
            header,
            info: tag::TextInformation {
                encoding: data[0],
                data: data[1..].to_vec(),
            },
        })),
        "APIC" => {
            let extracted_picture = extract::extract_picture(&data).unwrap();

            Ok(tag::Frame::Picture(tag::Id3v2PictureFrame {
                header,
                picture: tag::Picture {
                    encoding: extracted_picture.encoding,
                    mime: extracted_picture.mime,
                    picture_type: extracted_picture.picture_type,
                    description: extracted_picture.description,
                    data: extracted_picture.data,
                },
            }))
        }
        _ => Err(format!("Unknown frame id {}", ascii_id)),
    }
}

fn parse_frames(bytes: &Vec<u8>) -> Vec<tag::Frame> {
    let frame_bytes = bytes.clone();

    let mut idx = 0;
    let mut frames: Vec<tag::Frame> = vec![];

    while idx < frame_bytes.len() {
        // There are no frame identifiers with 0x00 0x00 0x00 0x00
        // therefore it is padding and end of frame bytes
        if frame_bytes[idx] == 0x00
            && frame_bytes[idx + 1] == 0x00
            && frame_bytes[idx + 2] == 0x00
            && frame_bytes[idx + 3] == 0x00
        {
            break;
        }

        let (unparsed_frame_bytes, end) = extract::extract_frame(idx, &frame_bytes);

        frames.push(parse_frame(&unparsed_frame_bytes).unwrap());
        idx += end + 1;
    }

    frames
}
