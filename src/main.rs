use clap::Parser;
use std::{ffi::OsStr, fs, path, time::Instant};

mod extract;
mod parse;
mod tag;
mod utility;

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

    /// Description of cover art image
    #[arg(short, long)]
    description: Option<String>,

    /// Path to the MP3 file
    #[arg(short, long)]
    input_file: Option<String>,

    /// Path to output tagged file
    #[arg(short, long)]
    output_file: Option<String>,

    /// Folder with files to tag (MP3 files only), will override --input_file (-i) and --output_file (-o)
    #[arg(long)]
    folder_input: Option<String>,

    /// Folder to output results of all the files in folder_input
    #[arg(long)]
    folder_output: Option<String>,

    /// Reuse the filename as the title of the track (ignores -t and --track)
    #[arg(long)]
    reuse: bool,
}

fn process_folder(args: &mut Args) {
    let now = Instant::now();
    if let Some(folder_path) = &args.folder_input {
        println!("Processing folder: {}", folder_path);

        if let Some(output_folder) = &args.folder_output {
            let input_path = match folder_path.strip_suffix("/") {
                Some(x) => x,
                None => folder_path,
            };

            let output_path = match output_folder.strip_suffix("/") {
                Some(x) => x,
                None => output_folder,
            };

            fs::create_dir_all(output_path).unwrap();

            for file in
                fs::read_dir(folder_path).expect("directory must be readable and accessible")
            {
                let file = file.expect("file must be valid and readable");

                println!(
                    "{}",
                    file.file_name()
                        .into_string()
                        .expect("must be readable file name")
                );

                args.input_file = Some(format!(
                    "{}/{}",
                    input_path,
                    file.file_name()
                        .into_string()
                        .expect("must be readable file name"),
                ));

                args.output_file = Some(
                    output_path.to_string()
                        + "/tagged-"
                        + &file
                            .file_name()
                            .into_string()
                            .expect("must be readable file name"),
                );

                if args.reuse {
                    let mut filename = file
                        .file_name()
                        .into_string()
                        .expect("must be readable username");

                    if let Some((left, _)) = filename.split_once(".") {
                        filename = left.to_string();
                    }

                    println!(
                        "Reusing filename as track title: {} (-r)",
                        file.file_name()
                            .into_string()
                            .expect("must be readable file name")
                    );

                    args.track = Some(filename);
                }

                process_single_file(args);
            }

            println!("All files successfully tagged.");
            println!("Total time elapsed: {}ms", now.elapsed().as_millis());

            return;
        }

        eprintln!("If attempting to tag all files in a folder, please include an output folder using --folder-output <PATH>");
    }
}

fn process_single_file(args: &Args) {
    if let Some(input) = &args.input_file {
        if let Some(output) = &args.output_file {
            println!("Processing file: {}", input);

            let now = Instant::now();

            let bytes = fs::read(input.to_string()).expect("must be readable file");

            let (id3v2_bytes, audio_data) = extract::extract_tag(&bytes);

            // println!("First Music Byte: {:#04X?}", audio_data[0]);

            let mut tag: tag::Id3v2Tag = match parse::parse_tag(&id3v2_bytes) {
                Ok(x) => x,
                Err(x) => {
                    eprintln!("{}", x);
                    return;
                }
            };

            if let Some(x) = &args.cover_art_path {
                if let Some(y) = &args.description {
                    let cover_art_bytes = fs::read(x).expect("must be readable file");

                    let file_extension = match path::Path::new(x)
                        .extension()
                        .and_then(OsStr::to_str)
                        .unwrap()
                    {
                        "jpg" => "jpg",
                        "png" => "png",
                        _ => {
                            eprintln!("cover art picture must be either a .jpg or .png file.");
                            return;
                        }
                    };

                    tag.set_cover_art(tag::Picture {
                        encoding: 0x03,
                        mime: "image/".to_owned() + file_extension + "\0",
                        picture_type: 0x03,
                        description: y.to_string() + "\0",
                        data: cover_art_bytes,
                    })
                    .unwrap();
                } else {
                    eprintln!("Must provide a description to embed an image");
                    return;
                }
            }
            // println!("cover art bytes size: {:?}", cover_art_bytes.len());

            if let Some(x) = &args.track {
                tag.set_song_title(x.to_string() + "\0").unwrap();
            }

            if let Some(x) = &args.name {
                tag.set_song_artist_name(x.to_string() + "\0").unwrap();
            }

            if let Some(x) = &args.album {
                tag.set_album_title(x.to_string() + "\0").unwrap();
            }

            if let Some(x) = &args.main_artist {
                tag.set_album_artist_name(x.to_string() + "\0").unwrap();
            }

            let _ = fs::write(output.clone(), [tag.into_bytes(), audio_data].concat());

            println!(
                "{:?} | File successfully tagged, saved to {}",
                now.elapsed(),
                output
            );

            return;
        }

        eprintln!("Must provide an output file to process: use -o <FILE> or --output-file <FILE>");
        return;
    }

    eprintln!("Must provide an input file to process");
}

fn main() {
    let mut args = Args::parse();

    if let Some(_) = args.folder_input {
        process_folder(&mut args);
    } else {
        process_single_file(&args);
    }
}
