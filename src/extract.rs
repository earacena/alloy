use crate::utility;
use crate::tag;

pub fn extract_tag(bytes: &Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    // add 10 to include header size
    let total_tag_size = utility::convert_safesynch_to_u32(bytes[6], bytes[7], bytes[8], bytes[9]) + 10;
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

    (
        bytes[..total_tag_size].to_vec(),
        bytes[(total_tag_size + 1)..].to_vec(),
    )
}

pub fn extract_picture(bytes: &Vec<u8>) -> Result<tag::Picture, String> {
    let mut encoding_byte: u8 = 0x03;
    let mut mime_bytes: Vec<u8> = vec![];
    let mut picture_type_byte = 0x03;
    let mut description_bytes: Vec<u8> = vec![];
    let mut data_bytes: Vec<u8> = vec![];

    let mut stage = "encoding";

    for byte in bytes.iter() {
        match stage {
            "encoding" => {
                encoding_byte = *byte;
                stage = "mime";
            }
            "mime" => {
                if *byte == 0x00 {
                    stage = "type";
                } else {
                    mime_bytes.push(*byte);
                }
            }
            "type" => {
                picture_type_byte = *byte;
                stage = "description";
            }
            "description" => {
                if *byte == 0x00 {
                    stage = "data";
                } else {
                    description_bytes.push(*byte);
                }
            }
            "data" => {
                data_bytes.push(*byte);
            }
            &_ => return Err("warning unknown stage while extracting picture".to_string()),
        }
    }

    Ok(tag::Picture {
        encoding: encoding_byte,
        mime: match String::from_utf8(mime_bytes).unwrap().as_str() {
            "image/png" => tag::MimeType::Png,
            "image/jpeg" => tag::MimeType::Jpeg,
            _ => tag::MimeType::Unknown,
        },
        picture_type: picture_type_byte,
        description: String::from_utf8(description_bytes).unwrap(),
        data: data_bytes,
    })
}

pub fn extract_frame(idx: usize, bytes: &Vec<u8>) -> (Vec<u8>, usize) {
    let total_frame_size = utility::convert_safesynch_to_u32(
        bytes[idx + 4],
        bytes[idx + 5],
        bytes[idx + 6],
        bytes[idx + 7],
    ) + 10;

    let start = idx;
    let end = idx + usize::try_from(total_frame_size).unwrap();

    return (bytes[start..end].to_vec(), end);
}
