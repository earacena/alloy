use core::fmt;
use std::mem;

use crate::utility::{self, convert_u32_to_safesynch};

#[derive(Debug)]
pub enum Frame {
    Text(Id3v2TextFrame),
    Picture(Id3v2PictureFrame),
}

#[derive(Debug)]
pub struct Id3v2Header {
    pub(crate) identifier: [u8; 3],
    pub(crate) version: [u8; 2],
    pub(crate) flags: u8,
    pub(crate) size: u32,
}

impl Id3v2Header {
    fn into_bytes(&self) -> Vec<u8> {
        let identifier_bytes = self.identifier.to_vec();
        let version_bytes = self.version.to_vec();
        let flag_bytes = vec![self.flags];
        let size_bytes = utility::convert_u32_to_safesynch(self.size).to_vec();

        [identifier_bytes, version_bytes, flag_bytes, size_bytes].concat()
    }
}

#[derive(Debug)]
pub struct Id3v2ExtendedHeader {
    pub(crate) size: u32,
    pub(crate) number_of_flag_bytes: u8,
    pub(crate) flags: u8,
    pub(crate) b_flag_length: u8,
    pub(crate) c_flag_length: u8,
    pub(crate) total_frame_crc: Option<u64>,
    pub(crate) d_flag_length: u8,
    pub(crate) restrictions: Option<u8>,
}

impl Id3v2ExtendedHeader {
    fn into_bytes(&self) -> Vec<u8> {
        let size_bytes = utility::convert_u32_to_safesynch(self.size).to_vec();
        let number_of_flag_bytes_vec = vec![self.number_of_flag_bytes];
        let flag_byte = vec![self.flags];
        let b_flag_length_byte = vec![self.b_flag_length];
        let c_flag_length_byte = vec![self.c_flag_length];
        let crc_bytes = match self.total_frame_crc {
            Some(crc) => utility::convert_u64_to_safesynch(crc).to_vec(),
            None => vec![],
        };
        let d_flag_length_byte = vec![self.d_flag_length];
        let restrictions = match self.restrictions {
            Some(r) => vec![r],
            None => vec![],
        };

        [
            size_bytes,
            number_of_flag_bytes_vec,
            flag_byte,
            b_flag_length_byte,
            c_flag_length_byte,
            crc_bytes,
            d_flag_length_byte,
            restrictions,
        ]
        .concat()
    }
}

#[derive(Debug)]
pub struct Id3v2FrameHeader {
    pub(crate) identifier: [u8; 4],
    pub(crate) size: u32, // 4 bytes representing a 32 bit safesynch integer
    pub(crate) flags: [u8; 2],
}

impl Id3v2FrameHeader {
    fn id_str(&self) -> String {
        String::from_utf8(self.identifier.to_vec()).unwrap()
    }

    fn into_bytes(&self) -> Vec<u8> {
        let identifier_bytes = self.identifier.to_vec();
        let size_bytes = utility::convert_u32_to_safesynch(self.size).to_vec();
        let flag_bytes = self.flags.to_vec();

        [identifier_bytes, size_bytes, flag_bytes].concat()
    }
}

#[derive(Debug)]
pub struct Id3v2TextFrame {
    pub(crate) header: Id3v2FrameHeader,
    pub(crate) info: TextInformation,
}

impl Id3v2TextFrame {
    fn into_bytes(&self) -> Vec<u8> {
        [self.header.into_bytes(), self.info.into_bytes()].concat()
    }
}

pub struct TextInformation {
    pub(crate) encoding: u8,
    pub(crate) data: Vec<u8>,
}

impl fmt::Debug for TextInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TextInformation [{} '{}']",
            self.encoding,
            String::from_utf8(self.data.clone()).unwrap()
        )
    }
}

impl TextInformation {
    fn into_bytes(&self) -> Vec<u8> {
        [vec![self.encoding], self.data.clone()].concat()
    }
}

#[derive(Debug)]
pub struct Id3v2PictureFrame {
    pub(crate) header: Id3v2FrameHeader,
    pub(crate) picture: Picture,
}

impl Id3v2PictureFrame {
    fn into_bytes(&self) -> Vec<u8> {
        let header_bytes = self.header.into_bytes();
        let picture_bytes = self.picture.into_bytes();

        [header_bytes, picture_bytes].concat()
    }
}

pub struct Picture {
    pub(crate) encoding: u8, // 0x03 for utf-8
    pub(crate) mime: String,
    pub(crate) picture_type: u8, // 0x03 for cover art
    pub(crate) description: String,
    pub(crate) data: Vec<u8>,
}

impl fmt::Debug for Picture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Picture [{} '{}' {} '{}' first five: {:?} last five: {:?}]",
            self.encoding,
            self.mime,
            self.picture_type,
            self.description,
            self.data[..5].to_vec(),
            self.data[self.data.len() - 5..].to_vec(),
        )
    }
}

impl Picture {
    fn into_bytes(&self) -> Vec<u8> {
        let description_bytes = self.description.clone().into_bytes();
        let mime_bytes = self.mime.clone().into_bytes();

        [
            vec![self.encoding],
            mime_bytes,
            vec![self.picture_type],
            description_bytes,
            self.data.clone(),
        ]
        .concat()
    }

    fn size(&self) -> usize {
        self.into_bytes().len()
    }
}

#[derive(Debug)]
pub struct Id3v2Tag {
    pub(crate) header: Id3v2Header,
    pub(crate) extended_header: Option<Id3v2ExtendedHeader>,
    pub(crate) frames: Vec<Frame>,
    pub(crate) footer: Option<Id3v2Header>,
}

impl Id3v2Tag {
    fn new_text_frame(&mut self, frame_id: &str, encoding: u8, data: Vec<u8>) -> Id3v2TextFrame {
        let id_bytes = frame_id.as_bytes();
        let new_frame = Id3v2TextFrame {
            // size has an additional byte for encoding
            header: Id3v2FrameHeader {
                identifier: [id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3]],
                size: u32::try_from(data.len()).unwrap() + 1,
                flags: [0x00, 0x00],
            },
            info: TextInformation { encoding, data },
        };

        new_frame
    }

    fn new_attached_picture_frame(&mut self, picture: Picture) -> Id3v2PictureFrame {
        let id_bytes = "APIC".to_string().into_bytes();

        Id3v2PictureFrame {
            header: Id3v2FrameHeader {
                identifier: [id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3]],
                size: u32::try_from(picture.size()).unwrap(),
                flags: [0x00, 0x00],
            },
            picture: Picture {
                encoding: picture.encoding,
                mime: picture.mime,
                picture_type: 0x03,
                description: picture.description,
                data: picture.data,
            },
        }
    }

    fn set_text_frame(&mut self, frame_id: &str, data: String) -> Result<(), String> {
        // Find frame
        let frame_idx = self.frames.iter().position(|x| match x {
            Frame::Text(x) => x.header.id_str() == frame_id,
            _ => false,
        });

        if let Some(idx) = frame_idx {
            if let Frame::Text(prev_frame) = &self.frames[idx] {
                self.header.size -= u32::try_from(prev_frame.into_bytes().len()).unwrap();

                let data_bytes = data.into_bytes();
                let new_frame = self.new_text_frame(frame_id, 0x03, data_bytes);

                self.header.size += u32::try_from(new_frame.into_bytes().len()).unwrap();
                self.frames[idx] = Frame::Text(new_frame);
            }
        } else {
            let new_frame = Frame::Text(self.new_text_frame(frame_id, 0x03, data.into_bytes()));
            self.header.size += u32::try_from(match &new_frame {
                Frame::Text(x) => x.into_bytes().len(),
                Frame::Picture(x) => x.into_bytes().len(),
            })
            .unwrap();
            self.frames.push(new_frame);
        }

        Ok(())
    }

    fn set_attached_picture_frame(&mut self, picture: Picture) -> Result<(), String> {
        let frame_idx = self.frames.iter().position(|x| match x {
            Frame::Text(x) => x.header.id_str() == "APIC",
            _ => false,
        });

        if let Some(idx) = frame_idx {
            if let Frame::Picture(prev_frame) = &self.frames[idx] {
                self.header.size -= u32::try_from(prev_frame.picture.size()).unwrap();
                self.header.size += u32::try_from(picture.size()).unwrap();

                let new_frame = Frame::Picture(self.new_attached_picture_frame(picture));
                self.frames[idx] = new_frame;
            } else {
                return Err("attempting to set a non-text frame as a picture frame".to_string());
            }
        } else {
            let new_frame = Frame::Picture(self.new_attached_picture_frame(picture));

            self.header.size += u32::try_from(match &new_frame {
                Frame::Text(x) => x.into_bytes().len(),
                Frame::Picture(x) => x.into_bytes().len(),
            })
            .unwrap();

            self.frames.push(new_frame);
        }

        Ok(())
    }

    pub fn set_song_title(&mut self, song_title: String) -> Result<(), String> {
        // TIT2 is song title
        match self.set_text_frame("TIT2", song_title) {
            Ok(()) => Ok(()),
            Err(x) => Err(x),
        }
    }

    pub fn set_song_artist_name(&mut self, song_artist_name: String) -> Result<(), String> {
        match self.set_text_frame("TPE1", song_artist_name) {
            Ok(()) => Ok(()),
            Err(x) => Err(x),
        }
    }

    pub fn set_album_title(&mut self, album_title: String) -> Result<(), String> {
        match self.set_text_frame("TALB", album_title) {
            Ok(()) => Ok(()),
            Err(x) => Err(x),
        }
    }

    pub fn set_album_artist_name(&mut self, album_artist_name: String) -> Result<(), String> {
        match self.set_text_frame("TPE2", album_artist_name) {
            Ok(()) => Ok(()),
            Err(x) => Err(x),
        }
    }

    pub fn set_cover_art(&mut self, picture: Picture) -> Result<(), String> {
        match self.set_attached_picture_frame(picture) {
            Ok(()) => Ok(()),
            Err(x) => Err(x),
        }
    }

    pub fn get_size(self) -> u64 {
        let mut total_tag_size = 0;

        // header fixed size
        total_tag_size += 10;

        if self.extended_header.is_some() {
            total_tag_size += self.extended_header.as_ref().unwrap().size + 10;
        }

        for frame in self.frames {
            total_tag_size += match frame {
                Frame::Picture(x) => x.header.size + 10,
                Frame::Text(x) => x.header.size + 10,
            };
        }

        if self.footer.is_some() {
            total_tag_size += 10;
        }

        total_tag_size.into()
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        // Return the stored information as a tag in bytes
        let header_bytes = self.header.into_bytes();
        let extended_header_bytes: Vec<u8> = match &self.extended_header {
            Some(e) => e.into_bytes(),
            None => vec![],
        };
        let mut frames_bytes: Vec<u8> = vec![];
        for frame in &self.frames {
            let mut bytes = match frame {
                Frame::Text(x) => x.into_bytes(),
                Frame::Picture(x) => x.into_bytes(),
            };

            frames_bytes.append(&mut bytes);
        }

        // -10 to account for header size that is not counted
        let mut total_size: i32 = -10;
        let footer_bytes: Vec<u8> = match &self.footer {
            Some(f) => {
                // Account for footer size
                total_size -= 10;
                (*f.into_bytes()).to_vec()
            }
            None => vec![],
        };

        let mut result = [
            header_bytes,
            extended_header_bytes,
            frames_bytes,
            footer_bytes,
        ]
        .concat();

        total_size += i32::try_from(result.len()).unwrap();
        // Ensure header size is accurate by updating total number of bytes
        let size_bytes = convert_u32_to_safesynch(u32::try_from(total_size).unwrap());
        // 6 7 8 9
        result[6] = size_bytes[0];
        result[7] = size_bytes[1];
        result[8] = size_bytes[2];
        result[9] = size_bytes[3];

        // for byte in &result {
        //     print!("{} ", byte);
        // }

        result
    }
}
