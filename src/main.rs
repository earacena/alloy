use std::{fmt::format, fs};

// An ID3v2 tag can be detected with the following pattern:
//      $49 44 33 yy yy xx zz zz zz zz
//    Where yy is less than $FF, xx is the 'flags' byte and zz is less than
//    $80.

#[derive(Debug)]
enum MimeType {
    Png,
    Jpeg,
    Unknown,
}

#[derive(Debug)]
enum Frame {
    Text(Id3v2TextFrame),
    Picture(Id3v2PictureFrame)
}

impl MimeType {
    fn to_string(&self) -> String {
        match self {
            MimeType::Jpeg => "image/jpeg".to_string(),
            MimeType::Png => "image/png".to_string(),
            MimeType::Unknown => "image/".to_string(),
        }
    }

    fn into_bytes(&self) -> Vec<u8> {
        let mut bytes = self.to_string().into_bytes();
        bytes.push(0x00);
        bytes
    }
}

#[derive(Debug)]
struct Id3v2Header {
    identifier: [u8; 3],
    version: [u8; 2],
    flags: u8,
    size: u32,
}

impl Id3v2Header {
    fn into_bytes(self) -> Vec<u8> {
        let identifier_bytes = self.identifier.to_vec();
        let version_bytes = self.version.to_vec();
        let flag_bytes = vec![self.flags];
        let size_bytes = convert_u32_to_safesynch(self.size).to_vec();

        [identifier_bytes, version_bytes, flag_bytes, size_bytes].concat()
    }
}

fn convert_u32_to_safesynch(value: u32) -> [u8; 4] {
    let byte0: u8 = u8::try_from((value & 0b00001111111000000000000000000000) >> 28).unwrap();
    let byte1: u8 = u8::try_from((value & 0b00000000000111111100000000000000) >> 14).unwrap();
    let byte2: u8 = u8::try_from((value & 0b00000000000000000011111110000000) >> 7).unwrap();
    let byte3: u8 = u8::try_from(value & 0b00000000000000000000000001111111).unwrap();

    [byte0, byte1, byte2, byte3]
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
struct Id3v2FrameHeader {
    identifier: [u8; 4],
    size: u32, // 4 bytes representing a 32 bit safesynch integer
    flags: [u8; 2],
}

impl Id3v2FrameHeader {
    fn id_str(&self) -> String {
        String::from_utf8(self.identifier.to_vec()).unwrap()
    }
}

#[derive(Debug)]
struct Id3v2TextFrame {
    header: Id3v2FrameHeader,
    data_encoding: Option<u8>, // some fields may encode a byte for encoding type
    data: Vec<u8>,
}

#[derive(Debug)]
struct Id3v2PictureFrame {
    header: Id3v2FrameHeader,
    picture: Picture
}

struct Id3v2Tag {
    header: Id3v2Header,
    extended_header: Option<Id3v2ExtendedHeader>,
    frames: Vec<Frame>,
    footer: Option<Id3v2Header>,
}

#[derive(Debug)]
struct Picture {
    encoding: u8,                   // 0x03 for utf-8
    mime: MimeType,
    picture_type: u8,               // 0x03 for cover art
    description: String,
    data: Vec<u8>,
}

impl Picture {
    fn into_bytes(&self) -> Vec<u8> {
        let mut description_bytes = self.description.clone().into_bytes();
        description_bytes.push(0x00);
        let mut mime_bytes = self.mime.to_string().into_bytes();
        mime_bytes.push(0x00);
        [mime_bytes, vec![self.picture_type], description_bytes.to_vec(), self.data.clone()].concat()
    }

    fn size(&self) -> usize {
        self.into_bytes().len()
    }
}

impl Id3v2Tag {
    fn find_frame(&mut self, frame_id: &str) -> Option<&mut Frame> {
        for frame in self.frames.iter_mut() {
            match frame {
                Frame::Text(x) => {
                    if x.header.id_str() == frame_id {
                        return Some(frame)
                    }
                },
                Frame::Picture(x) => {
                    if x.header.id_str() == frame_id {
                        return Some(frame);
                    }
                }
            }

            continue;
        }

        return None;
    }

    fn frame_exists(&self, frame_id: &str) -> bool {
        for frame in self.frames.iter() {
            match frame {
                Frame::Text(x) => {
                    if x.header.id_str() == frame_id {
                        return true;
                    }
                },
                Frame::Picture(x) => {
                    if x.header.id_str() == frame_id {
                        return true;
                    }
                }
            }

            continue;
        }

        return false;
    }

    fn new(id3v2_bytes: &Vec<u8>) -> Result<Self, String> {
        if id3v2_bytes[0] != 0x49 || id3v2_bytes[1] != 0x44 || id3v2_bytes[2] != 0x33 {
            // Not an ID3v2 tag
            return Err(format!(
                "not a valid ID3v2.4 tag ({:#04X?} {:#04X?} {:#04X?})",
                id3v2_bytes[0], id3v2_bytes[1], id3v2_bytes[2]
            ));
        }

        for byte in id3v2_bytes {
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

        // header is always 10 bytes
        // extended header might or might not be present
        // frames start after extended up to footer
        let frames_start = if extended_header.is_some() {
            usize::try_from(extended_header.as_ref().unwrap().size + 10).unwrap()
        } else {
            10
        };

        let frames_end = if footer_present {
            id3v2_bytes.len() - 10
        } else {
            id3v2_bytes.len()
        };

        let frame_bytes = &id3v2_bytes[frames_start..frames_end].to_vec();
        let frames = parse_frames(frame_bytes);
        let footer: Option<Id3v2Header> = if footer_present {
            Some(parse_header(&id3v2_bytes[frames_end + 1..].to_vec()))
        } else {
            None
        };

        println!("");
        println!("header: {:#04X?}", header);
        println!("extended header: {:#04X?}", extended_header);
        println!("frames: {:#04X?}", frames);
        println!("footer: {:#04X?}", footer);

        Ok(Self {
            header,
            extended_header,
            frames,
            footer,
        })
    }

    fn new_text_frame(&mut self, frame_id: &str, encoding: u8, data: Vec<u8>) -> Id3v2TextFrame {
        let id_bytes = frame_id.as_bytes();
        let new_frame = Id3v2TextFrame {
            header: Id3v2FrameHeader {
                // TIT2 is
                identifier: [id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3]],
                size: u32::try_from(data.len()).unwrap() + 1,
                flags: [0x00, 0x00],
            },
            data_encoding: Some(encoding),
            data,
        };

        new_frame
    }

    fn new_attached_picture_frame(&mut self, picture: Picture) -> Id3v2PictureFrame {
        let id_bytes = "APIC".to_string().into_bytes();
        let mime_bytes = picture.mime.into_bytes();

        Id3v2PictureFrame {
            header: Id3v2FrameHeader {
                identifier: [id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3]],
                size: u32::try_from(picture.data.len()).unwrap() + u32::try_from(picture.description.len()).unwrap() + u32::try_from(mime_bytes.len()).unwrap() + 2,
                flags: [0x00, 0x00],
            },
            picture: Picture {
                encoding: picture.encoding,
                mime: picture.mime,
                picture_type: 0x03,
                description: picture.description,
                data: picture.data,
            }

        }
    }

    fn set_text_frame(&mut self, frame_id: &str, data: String) -> Result<(), String> {
        let result = self.find_frame(frame_id);
        if let Some(x) = result {
            if let Frame::Text(frame) = x {
                let mut data_bytes = data.into_bytes();
                data_bytes.push(0x00);
                
                frame.header.size -= u32::try_from(frame.data.len()).unwrap();
                frame.header.size += u32::try_from(data_bytes.len()).unwrap();
                frame.data = data_bytes;
                frame.header.flags = [0x00, 0x00];
            } else {
                return Err("attempting to set a non-text frame as a picture frame".to_string());
            }
        } else {
            self.new_text_frame(frame_id, 0x03, data.into_bytes());
        }

        Ok(())
    }

    fn set_attached_picture_frame(
        &mut self,
        picture: Picture,
    ) -> Result<(), String> {

        let result = self.find_frame("APIC");
        if let Some(x) = result {
            if let Frame::Picture(frame) = x {
                frame.header.size -= u32::try_from(frame.picture.size()).unwrap();
                frame.header.size += u32::try_from(picture.size()).unwrap();

                frame.picture = picture;
            } else {
                return Err("attempting to set a non-text frame as a picture frame".to_string());
            }
        } else {
            self.new_attached_picture_frame(picture);
        }

        Ok(())
    }

    fn set_song_title(&mut self, song_title: String) -> Result<(), String> {
        // TIT2 is song title
        match self.set_text_frame("TIT2", song_title) {
            Ok(()) => Ok(()),
            Err(x) => Err(x),
        }
    }

    fn set_song_artist_name(&mut self, song_artist_name: String) -> Result<(), String> {
        match self.set_text_frame("TPE1", song_artist_name) {
            Ok(()) => Ok(()),
            Err(x) => Err(x),
        }
    }

    fn set_album_title(&mut self, album_title: String) -> Result<(), String> {
        match self.set_text_frame("TALB", album_title) {
            Ok(()) => Ok(()),
            Err(x) => Err(x),
        }
    }

    fn set_cover_art(
        &mut self,
        picture: Picture
    ) -> Result<(), String> {
        match self.set_attached_picture_frame(picture) {
            Ok(()) => Ok(()),
            Err(x) => Err(x),
        }
    }

    fn get_size(self) -> u64 {
        let mut total_tag_size = 0;

        // header fixed size
        total_tag_size += 10;

        if self.extended_header.is_some() {
            total_tag_size += self.extended_header.as_ref().unwrap().size + 10;
        }

        for frame in self.frames {
            total_tag_size += frame.header.size + 10;
        }

        if self.footer.is_some() {
            total_tag_size += 10;
        }

        total_tag_size.into()
    }

    fn as_bytes() -> Vec<u8> {
        // Return the stored information as a tag in bytes
        todo!()
    }
}

fn parse_tag(id3v2_bytes: &Vec<u8>) -> Result<Id3v2Tag, String> {
    if id3v2_bytes[0] != 0x49 || id3v2_bytes[1] != 0x44 || id3v2_bytes[2] != 0x33 {
        // Not an ID3v2 tag
        return Err(format!(
            "not a valid ID3v2.4 tag ({:#04X?} {:#04X?} {:#04X?})",
            id3v2_bytes[0], id3v2_bytes[1], id3v2_bytes[2]
        ));
    }

    for byte in id3v2_bytes {
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

    // header is always 10 bytes
    // extended header might or might not be present
    // frames start after extended up to footer
    let frames_start = if extended_header.is_some() {
        usize::try_from(extended_header.as_ref().unwrap().size + 10).unwrap()
    } else {
        10
    };

    let frames_end = if footer_present {
        id3v2_bytes.len() - 10
    } else {
        id3v2_bytes.len()
    };

    let frame_bytes = &id3v2_bytes[frames_start..frames_end].to_vec();
    let frames = parse_frames(frame_bytes);
    let footer: Option<Id3v2Header> = if footer_present {
        Some(parse_header(&id3v2_bytes[frames_end + 1..].to_vec()))
    } else {
        None
    };

    println!("");
    println!("header: {:#04X?}", header);
    println!("extended header: {:#04X?}", extended_header);

    let tag = Id3v2Tag {
        header,
        extended_header,
        frames,
        footer,
    };

    Ok(tag)
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

fn extract_tag(bytes: &Vec<u8>) -> (Vec<u8>, Vec<u8>) {
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

    (
        bytes[..total_tag_size].to_vec(),
        bytes[(total_tag_size + 1)..].to_vec(),
    )
}

fn extract_picture(encoding: u8, bytes: &Vec<u8>) -> Result<Picture, String> {
    let mut mime_bytes: Vec<u8> = vec![];
    let mut picture_type_byte = 0x03;
    let mut description_bytes: Vec<u8> = vec![];
    let mut data_bytes: Vec<u8> = vec![];

    let mut stage = "mime";

    for byte in bytes.iter() {
        match stage {
            "mime" => {
                if *byte == 0x00 {
                    stage = "type";
                } else {
                    mime_bytes.push(*byte);
                }
            },
            "type" => {
                picture_type_byte = *byte;
                stage = "description";
            },
            "description" => {
                if *byte == 0x00 {
                    stage = "data";
                } else {
                    description_bytes.push(*byte);
                }
            },
            "data" => {
                data_bytes.push(*byte);
            },
            &_ => {
                return Err("warning unknown stage while extracting picture".to_string())
            }

        }
    }

    Ok(Picture {
        encoding,
        mime: match String::from_utf8(mime_bytes).unwrap().as_str() {
            "image/png" => MimeType::Png,
            "image/jpeg" => MimeType::Jpeg,
            _ => MimeType::Unknown,
        },
        picture_type: picture_type_byte,
        description: String::from_utf8(description_bytes).unwrap(),
        data: data_bytes,
    })

}

fn parse_frame(bytes: &Vec<u8>) -> Result<Frame, String> {
    let identifier = [bytes[0], bytes[1], bytes[2], bytes[3]];
    let size = convert_safesynch_to_u32(bytes[4], bytes[5], bytes[6], bytes[7]);
    let flags = [bytes[8], bytes[9]];

    let header = Id3v2FrameHeader {
        identifier,
        size,
        flags,
    };

    let mut data = bytes[10..].to_vec();
    let data_encoding: Option<u8> = match data[0] {
        0x0 => Some(0x0),
        0x1 => Some(0x1),
        0x2 => Some(0x2),
        0x3 => Some(0x3),
        _ => None,
    };

    // Encoding byte (if present) shouldn't be with field data
    if data_encoding.is_some() {
        data = data[1..].to_vec();
    }

    let binding = String::from_utf8(identifier.to_vec()).unwrap();
    let ascii_id = binding.as_str();

    match ascii_id {
        "TIT2" | "TALB" | "TPE1" => Ok(Frame::Text(Id3v2TextFrame {
            header,
            data_encoding,
            data,
        })),
        "APIC" => {
            let extracted_picture = extract_picture(data_encoding.unwrap(), &data).unwrap();
            
            Ok(Frame::Picture(Id3v2PictureFrame {
                header,
                picture: Picture {
                    encoding: data_encoding.unwrap(),
                    mime: extracted_picture.mime,
                    picture_type: extracted_picture.picture_type,
                    description: extracted_picture.description,
                    data: extracted_picture.data,
                }
            }))
        },
        _ => {
            Err(format!("Unknown frame id {}", ascii_id))
        }
    }
}

fn parse_frames(bytes: &Vec<u8>) -> Vec<Frame> {
    let frame_bytes = bytes.clone();

    let mut idx = 0;
    let mut frames: Vec<Frame> = vec![];

    while idx < frame_bytes.len() {
        let byte0 = frame_bytes[idx];
        let byte1 = frame_bytes[idx + 1];
        let byte2 = frame_bytes[idx + 2];
        let byte3 = frame_bytes[idx + 3];

        // There are no frame identifiers with 0x00 0x00 0x00 0x00
        // therefore it is padding and end of frames
        if byte0 == 0x00 && byte1 == 0x00 && byte2 == 0x00 && byte3 == 0x00 {
            break;
        }

        let total_frame_size = convert_safesynch_to_u32(
            frame_bytes[idx + 4],
            frame_bytes[idx + 5],
            frame_bytes[idx + 6],
            frame_bytes[idx + 7],
        ) + 10;

        let start = idx;
        let end = idx + usize::try_from(total_frame_size).unwrap();

        let unparsed_frame_bytes = &frame_bytes[start..end].to_vec();

        frames.push(parse_frame(unparsed_frame_bytes).unwrap());
        idx += end + 1;
    }

    frames
}

fn get_field_name(identifier: [u8; 4]) -> String {
    let binding = String::from_utf8(identifier.to_vec()).unwrap();
    let ascii_id = binding.as_str();

    match ascii_id {
        "AENC" => "Audio encryption".to_string(),
        "APIC" => "Attached picture".to_string(),
        "ASPI" => "Audio seek point index".to_string(),
        "COMM" => "Comments".to_string(),
        "COMR" => "Commercial frame".to_string(),

        "ENCR" => "Encryption method registration".to_string(),
        "EQU2" => "Equalisation (2)".to_string(),
        "ETCO" => "Event timing codes".to_string(),

        "GEOB" => "General encapsulated object".to_string(),
        "GRID" => "Group identification registration".to_string(),

        "LINK" => "Linked information".to_string(),

        "MCDI" => "Music CD identifier".to_string(),
        "MLLT" => "MPEG location lookup table".to_string(),

        "OWNE" => "Ownership frame".to_string(),

        "PRIV" => "Private frame".to_string(),
        "PCNT" => "Play counter".to_string(),
        "POPM" => "Popularimeter".to_string(),
        "POSS" => "Position synchronisation frame".to_string(),

        "RBUF" => "Recommended buffer size".to_string(),
        "RVA2" => "Relative volume adjustment (2)".to_string(),
        "RVRB" => "Reverb".to_string(),

        "SEEK" => "Seek frame".to_string(),
        "SIGN" => "Signature frame".to_string(),
        "SYLT" => "Synchronised lyric/text".to_string(),
        "SYTC" => "Synchronised tempo codes".to_string(),

        "TALB" => "Album/Movie/Show title".to_string(),
        "TBPM" => "BPM (beats per minute)".to_string(),
        "TCOM" => "Composer".to_string(),
        "TCON" => "Content type".to_string(),
        "TCOP" => "Copyright message".to_string(),
        "TDEN" => "Encoding time".to_string(),
        "TDLY" => "Playlist delay".to_string(),
        "TDOR" => "Original release time".to_string(),
        "TDRC" => "Recording time".to_string(),
        "TDRL" => "Release time".to_string(),
        "TDTG" => "Tagging time".to_string(),
        "TENC" => "Encoded by".to_string(),
        "TEXT" => "Lyricist/Text writer".to_string(),
        "TFLT" => "File type".to_string(),
        "TIPL" => "Involved people list".to_string(),
        "TIT1" => "Content group description".to_string(),
        "TIT2" => "Title/songname/content description".to_string(),
        "TIT3" => "Subtitle/Description refinement".to_string(),
        "TKEY" => "Initial key".to_string(),
        "TLAN" => "Language(s)".to_string(),
        "TLEN" => "Length".to_string(),
        "TMCL" => "Musician credits list".to_string(),
        "TMED" => "Media type".to_string(),
        "TMOO" => "Mood".to_string(),
        "TOAL" => "Original album/movie/show title".to_string(),
        "TOFN" => "Original filename".to_string(),
        "TOLY" => "Original lyricist(s)/text writer(s)".to_string(),
        "TOPE" => "Original artist(s)/performer(s)".to_string(),
        "TOWN" => "File owner/licensee".to_string(),
        "TPE1" => "Lead performer(s)/Soloist(s)".to_string(),
        "TPE2" => "Band/orchestra/accompaniment".to_string(),
        "TPE3" => "Conductor/performer refinement".to_string(),
        "TPE4" => "Interpreted, remixed, or otherwise modified by".to_string(),
        "TPOS" => "Part of a set".to_string(),
        "TPRO" => "Produced notice".to_string(),
        "TPUB" => "Publisher".to_string(),
        "TRCK" => "Track number/Position in set".to_string(),
        "TRSN" => "Internet radio station name".to_string(),
        "TRSO" => "Internet radio station owner".to_string(),
        "TSOA" => "Album sort order".to_string(),
        "TSOP" => "Performer sort order".to_string(),
        "TSOT" => "Title sort order".to_string(),
        "TSRC" => "ISRC (international standard recording code)".to_string(),
        "TSSE" => "Software/Hardware and settings used for encoding".to_string(),
        "TSST" => "Set subtitle".to_string(),
        // "TXXX" => "User defined text information frame".to_string(),
        "UFID" => "Unique file identifier".to_string(),
        "USER" => "Terms of use".to_string(),
        "USLT" => "Unsynchronised lyric/text transcription".to_string(),

        "WCOM" => "Commercial information".to_string(),
        "WCOP" => "Copyright/Legal information".to_string(),
        "WOAF" => "Official audio file webpage".to_string(),
        "WOAR" => "Official artist/performer webpage".to_string(),
        "WOAS" => "Official audio source webpage".to_string(),
        "WORS" => "Official Internet radio station homepage".to_string(),
        "WPAY" => "Payment".to_string(),
        "WPUB" => "Publishers official webpage".to_string(),
        // "WXXX" => "User defined URL link frame".to_string(),
        _ => "Unknown frame".to_string(),
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("must give a path");
    let bytes = fs::read(path).expect("unable to read file");

    let (id3v2_bytes, audio_data) = extract_tag(&bytes);

    let mut tag: Id3v2Tag = match Id3v2Tag::new(&id3v2_bytes) {
        Ok(x) => x,
        Err(x) => {
            println!("{}", x);
            return;
        }
    };

    let cover_art_path = "./test.png";
    let cover_art_bytes = fs::read(cover_art_path).expect("unable to read file");

    tag.set_song_title("Tag of Test".to_string());
    tag.set_song_artist("MC Test".to_string());
    tag.set_album_title("Testphonics".to_string());
    tag.set_album_artist("MC Test".to_string());
    tag.set_cover_art(
        cover_art_bytes,
        "A 300x300 picture with the word 'test' in the center".to_string(),
        MimeType::Jpeg,
    );

    // println!("{}", tag.as_bytes());
}
