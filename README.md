# alloy

## Description

A command-line tag editor for parsing, modifying, and writing ID3v2 metadata in MP3 files, written in Rust.

### Features

#### ID3 Parsing Support

* [x] ID3v2.4

#### ID3 Frame Support (ID3v2.4)

* [x] APIC - Attached picture
* [x] TIT2 - Title/songname/content description
* [x] TALB - Album/Movie/Show title
* [x] TPE1 - Lead performer(s)/Soloist(s)
* [x] TSSE - Software/Hardware and settings used for encoding
* [ ] TDRL - Release time

#### Parsing

* [x] Extract tags from given MP3 files
* [x] Output MP3 file with modified tag data

#### Editing

* [x] Modify supported frames in tags
* [x] Command-line interface
  * [x] Single file editing
  * [x] Bulk editing

### Usage

#### Download

Clone the repository to the desired directory using [Git](https://git-scm.com/):

```bash
cd ~/folder
git clone https://github.com/earacena/alloy.git
cd alloy/
```

#### Build

Build using [Cargo](https://rustup.rs/):

```bash
cargo build
```

Run the executable directly:

```bash
./target/debug/alloy
```

Or, use Cargo to run the executable:

```bash
cargo run
```

#### Example Usage

To view all available commands, run the executable directly:

```bash
alloy --help
```

Or using Cargo in the project directory, passing arguments with ```--```:

```bash
cargo run -- --help
```

##### Single file

```bash
alloy --input-file "~/path/to/file.mp3" --output-file "~/path/to/output.mp3" -t "Track title" -n "Track artist" -a "Album title" -c "~/path/to/art.jpg" -d "description of picture"
```

Note: ```--reuse``` flag uses the name of the file (excluding extension) as the name of the track, ignoring what is passed to ```-t``` or ```--track``` arguments.

##### Multiple files

To tag multiple files, ensure that the files are in a folder containing only MP3 files, and tagging follows the same process as single file tagging:

```bash
alloy --folder-input "~/path/to/folder" --folder-output "~/path/to/output/folder" -n "Example artist" -a "Example album" --reuse -c "~/path/to/art.jpg" -d "art description"
```

### Disclaimer

This is a work-in-progress tool, always make sure to backup all files before modifying them with this tool to prevent the risk of data corruption or loss. By using this tool, you acknowledge this risk and accept that I am not responsible for any and all data corruption or loss that may occur.
