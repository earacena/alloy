pub fn convert_safesynch_to_u32(byte0: u8, byte1: u8, byte2: u8, byte3: u8) -> u32 {
    u32::from(byte0) << 21 | u32::from(byte1) << 14 | u32::from(byte2) << 7 | u32::from(byte3)
}

pub fn convert_safesynch_to_u64(byte0: u8, byte1: u8, byte2: u8, byte3: u8, byte4: u8) -> u64 {
    u64::from(byte0) << 28
        | u64::from(byte1) << 21
        | u64::from(byte2) << 14
        | u64::from(byte3) << 7
        | u64::from(byte4)
}

pub fn convert_u32_to_safesynch(value: u32) -> [u8; 4] {
    let byte0: u8 = u8::try_from((value & 0b00001111111000000000000000000000) >> 21).unwrap();
    let byte1: u8 = u8::try_from((value & 0b00000000000111111100000000000000) >> 14).unwrap();
    let byte2: u8 = u8::try_from((value & 0b00000000000000000011111110000000) >> 7).unwrap();
    let byte3: u8 = u8::try_from(value & 0b00000000000000000000000001111111).unwrap();

    [byte0, byte1, byte2, byte3]
}

pub fn convert_u64_to_safesynch(value: u64) -> [u8; 5] {
    let byte0: u8 = u8::try_from((value & 0b11111110000000000000000000000000000) >> 28).unwrap();
    let byte1: u8 = u8::try_from((value & 0b00001111111000000000000000000000) >> 21).unwrap();
    let byte2: u8 = u8::try_from((value & 0b00000000000111111100000000000000) >> 14).unwrap();
    let byte3: u8 = u8::try_from((value & 0b00000000000000000011111110000000) >> 7).unwrap();
    let byte4: u8 = u8::try_from(value & 0b00000000000000000000000001111111).unwrap();

    [byte0, byte1, byte2, byte3, byte4]
}

pub fn get_field_name(identifier: [u8; 4]) -> String {
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