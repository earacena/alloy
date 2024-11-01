use std::{fs, time::Instant};
use clap::Parser;

mod parse;
mod utility;
mod tag;
mod extract;

/// A tag editor for parsing, modifying, and writing ID3 metadata in MP3 files, written in Rust.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Title of the song
    #[arg(short, long)]
    track: Option<String>,

    /// Name of the song's artist
    #[arg(short, long)]
    name: Option<String>,

    /// Title of the album 
    #[arg(short, long)]
    album: Option<String>,

    /// Name of the album's artist
    #[arg(short, long)]
    main_artist: Option<String>,

    /// Path to the cover art image of the song
    #[arg(short, long)]
    cover_art_path: Option<String>,

    // Description of cover art image
    #[arg(short, long)]
    description: Option<String>,

    /// Path to the MP3 file
    #[arg(short, long)]
    input_path: String,

    /// Path to output tagged file
    #[arg(short, long, default_value_t = String::from("."))]
    output_path: String,
}

// An ID3v2 tag can be detected with the following pattern:
//      $49 44 33 yy yy xx zz zz zz zz
//    Where yy is less than $FF, xx is the 'flags' byte and zz is less than
//    $80.
fn main() {
    let args = Args::parse();
    
    let now = Instant::now();

    let bytes = fs::read(args.input_path).expect("must be readable file");
    
    let (id3v2_bytes, audio_data) = extract::extract_tag(&bytes);

    // println!("First Music Byte: {:#04X?}", audio_data[0]);

    let mut tag: tag::Id3v2Tag = match parse::parse_tag(&id3v2_bytes) {
        Ok(x) => x,
        Err(x) => {
            println!("{}", x);
            return;
        }
    };

    if let Some(x) = args.cover_art_path {
        if let Some(y) = args.description {
            let cover_art_bytes = fs::read(x).expect("must be readable file");
            tag.set_cover_art(
                tag::Picture {
                    encoding: 0x03,
                    mime: tag::MimeType::Jpeg,
                    picture_type: 0x03,
                    description: y,
                    data: cover_art_bytes,
                }
            ).unwrap();
        } else {
            println!("Must provide a description to embed an image");
            return;
        }
    }
    // println!("cover art bytes size: {:?}", cover_art_bytes.len());

    if let Some(x) = args.track {
        tag.set_song_title(x).unwrap();
    }

    if let Some(x) = args.name {
        tag.set_song_artist_name(x).unwrap();
    }

    if let Some(x) = args.album {
        tag.set_album_title(x).unwrap();
    }

    if let Some(x) = args.main_artist {
        tag.set_album_artist_name(x).unwrap();
    }

    let _ = fs::write(args.output_path.clone(), [tag.into_bytes(), audio_data].concat());

    println!("\n");
    println!("Time elapsed: {}ms", now.elapsed().as_millis());
    println!("File successfully tagged, output saved to path: {}", args.output_path);
}   
