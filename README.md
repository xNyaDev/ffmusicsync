# ffmusicsync

A simple utility which creates an encoded music folder out of your library and keeps it updated using as least ffmpeg 
runs as possible.

## Requirements
- [ffmpeg](https://ffmpeg.org/) installed and added to PATH
- [rclone](https://rclone.org/) installed and added to PATH (Optional, only when using remotes)

## Features
- Encode an entire folder, copying already encoded songs
- Keep the folder updated with subsequent runs - you can remove and add songs to the original one
- Remove brackets from the filenames - no more `(Original Mix)` in every single name, customizable
- Lots of file formats supported - acts as a ffmpeg wrapper, supports everything ffmpeg does
- Keep a list of the encoded files so even if you change the renaming settings it won't need to re-encode them
- Copy covers to files post-encode*
- Support files being on rclone remotes - automatically downloads them, encodes locally and uploads

*See the [lofty crate](https://docs.rs/lofty/0.6.3/lofty/index.html#supported-formats) for a list of supported formats 
to copy from/to

## Configuration examples
`config.json` can contain comments, so you can just copy-paste the example.
```jsonc
{
  // Input and output directory
  // Can be either a string or an object:
  // If it's a string and contains a ':', it will be treated as an rclone remote
  "inputDirectory": "MyStorage:FLAC",
  "outputDirectory": {
    // Both fields are optional
    // If remote is not present or blank it will be treated as a local location 
    "remote": "MyStorage",
    "path": "Encoded"
  },
  // Optional, used only when either the input or output directory is remote
  "tempDirectory": "temp",
  // An array of all extensions that will be encoded
  // All that are not present here (such as mp3 in this case) will be just copied to the output directory
  "extensionsToEncode": [
    "flac"
  ],
  // A string containing the extension for encoded files
  "encodedExtension": "ogg",
  // Add covers to files after encoding them
  // ffmpeg can't do that by itself for OGG files
  // The option doesn't need to be present, defaults to false
  "copyCovers": true,
  // A string containing the ffmpeg params
  // ffmpeg command looks like:
  // ffmpeg -i <INPUT> <PARAMS> <OUTPUT>
  "ffmpegParams": "-c:a libopus -b:a 128K -vn",
  // Whether to remove brackets, the options don't need to be present, 
  // in which case it will behave the same way as if they were set to false
  // When set to true it will remove everything in between the brackets as well as one leading/trailing space
  // Example with the below config:
  // [LABEL] Author - Title (Original Mix) [YEAR].mp3
  // Would be renamed to:
  // Author - Title (Original Mix).mp3
  "removeRoundBrackets": false, // ()
  "removeSquareBrackets": true, // []
  "removeCurlyBrackets": false, // {}
  "removeAngleBrackets": false, // <>
}
```

## Command-line arguments
- `-c`, `--config` - Specify the config file (default: config.json)
- `--color` - Force colors to be enabled
- `--dry-run` - Do a trial run with no actual changes
- `-e`, `--encoded` - Specify the file storing info which songs are already encoded (default: encoded.json)
- `-h`, `--help` - Print help information
- `-q`, `--quiet` - Suppress ffmpeg output
- `-V`, `--version` - Print version information
- `-y`, `--yes` - Always assume "yes" as the answer to all prompts and run non-interactively

## Planned features
- [ViSQOL](https://github.com/google/visqol) support for automatic bitrate mode

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
