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
* [ ] Command-line interface
  * [ ] Single file editing
  * [ ] Bulk editing

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

TBD
