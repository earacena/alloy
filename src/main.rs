use std::fs;

mod parse;
mod utility;
mod tag;
mod extract;

// An ID3v2 tag can be detected with the following pattern:
//      $49 44 33 yy yy xx zz zz zz zz
//    Where yy is less than $FF, xx is the 'flags' byte and zz is less than
//    $80.
fn main() {
    let path = std::env::args().nth(1).expect("path must be given");
    let bytes = fs::read(path.clone()).expect("unable to read file");

    let (id3v2_bytes, audio_data) = extract::extract_tag(&bytes);

    println!("First Music Byte: {:#04X?}", audio_data[0]);

    let mut tag: tag::Id3v2Tag = match parse::parse_tag(&id3v2_bytes) {
        Ok(x) => x,
        Err(x) => {
            println!("{}", x);
            return;
        }
    };

    let cover_art_path = "./test.jpg";
    let cover_art_bytes = fs::read(cover_art_path).expect("file must exist");
    println!("cover art bytes size: {:?}", cover_art_bytes.len());

    tag.set_song_title("Tag of Test".to_string()).unwrap();
    tag.set_song_artist_name("MC Test".to_string()).unwrap();
    tag.set_album_title("Testphonics".to_string()).unwrap();
    tag.set_album_artist_name("MC Test".to_string()).unwrap();
    tag.set_cover_art(
        tag::Picture {
            encoding: 0x03,
            mime: tag::MimeType::Jpeg,
            picture_type: 0x03,
            description: "a 300 pixel by 300 pixel picture with a white background and the word 'test' in the center".to_string(),
            data: cover_art_bytes,
        }
    ).unwrap();

    println!("{:#?}", tag);
    //println!("Result: {:#?}", tag.into_bytes());

    let tagged_path = "./tagged.mp3";
    let _ = fs::write(tagged_path, [tag.into_bytes(), audio_data].concat());
}
