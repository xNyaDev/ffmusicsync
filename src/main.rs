use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::process::Command;

use clap::Parser;
use console::{set_colors_enabled, set_colors_enabled_stderr, Style};
use dialoguer::Confirm;
use json_comments::StripComments;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;

use crate::ogg_cover::copy_pictures;

mod tests;
mod ogg_cover;

/// A simple utility which creates an encoded music folder out of your library and keeps it updated
/// using as least ffmpeg runs as possible.
/// Requires ffmpeg to be installed and in PATH
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Specify the config file - See README.md for examples
    #[clap(short, long, default_value = "config.json")]
    config: String,
    /// Specify the file storing info which songs are already encoded
    #[clap(short, long, default_value = "encoded.json")]
    encoded: String,
    /// Force colors to be enabled
    #[clap(long)]
    color: bool,
    /// Always assume "yes" as the answer to all prompts and run non-interactively
    #[clap(short, long)]
    yes: bool,
    /// Suppress ffmpeg output
    #[clap(short, long)]
    quiet: bool,
    /// Do a trial run with no actual changes
    #[clap(long)]
    dry_run: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CLion does not auto-detect args type
    let args: Args = Args::parse();

    // Enable colors when running in a console or --color was passed
    if args.color {
        set_colors_enabled(true);
        set_colors_enabled_stderr(true);
    }

    // Styles used
    let bold_red = Style::new().bold().red();
    let bold_green = Style::new().bold().green();

    // Fail if the config file does not exist
    if !Path::new(&args.config).exists() {
        eprintln!("{}", bold_red.apply_to("Config file not found"));
        std::process::exit(1);
    }

    // Fail if ffmpeg is not found
    let ffmpeg_test = Command::new("ffmpeg").arg("-version").output();
    if ffmpeg_test.is_err() {
        eprintln!("{}", bold_red.apply_to("ffmpeg not found"));
        std::process::exit(1);
    }

    // Read config from file
    let config_file = File::open(&args.config)?;
    let config_reader = BufReader::new(config_file);
    let config_reader_no_comments = StripComments::new(config_reader);
    let config: Config = serde_json::from_reader(config_reader_no_comments)?;

    // Read already processed songs
    let mut encoded: HashMap<String, String> = if let Ok(encoded_file) = File::open(&args.encoded) {
        let encoded_reader = BufReader::new(encoded_file);
        serde_json::from_reader(encoded_reader)?
    } else {
        // None are processed if the file doesn't exist
        HashMap::new()
    };

    // Read songs that are present in the filesystem already
    let input = fs::read_dir(&config.input_directory)?
        .map(|file| file.unwrap().file_name().to_str().unwrap().to_string())
        .collect::<HashSet<String>>();
    let output = fs::read_dir(&config.output_directory)?
        .map(|file| file.unwrap().file_name().to_str().unwrap().to_string())
        .collect::<HashSet<String>>();

    // Check for name collisions
    let encoded_names = input
        .iter()
        .map(|input_file_name| create_output_file_name(input_file_name.to_string(), &config))
        .collect::<HashSet<String>>();
    if encoded_names.len() != input.len() {
        eprintln!(
            "{}",
            bold_red.apply_to("Found a name collision with the current settings, aborting")
        );
        let encoded = create_final_encoded_map(input, &config);
        // Find and print the colliding names
        encoded_names
            .into_iter()
            .for_each(|search_output_file_name| {
                let duplicates = encoded
                    .clone()
                    .into_iter()
                    .filter_map(|(input_file_name, output_file_name)| {
                        if output_file_name == search_output_file_name {
                            Some(input_file_name)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>();
                if duplicates.len() != 1 {
                    eprintln!(
                        "{} is the resulting file name for:",
                        search_output_file_name
                    );
                    duplicates.into_iter().for_each(|file_name| {
                        eprintln!(" - {}", file_name);
                    });
                    eprintln!();
                }
            });
        std::process::exit(2);
    }

    // Find which songs need to be processed
    let encoded_output = encoded.values().cloned().collect::<HashSet<String>>();
    let removed_output = encoded_output
        .difference(&output)
        .cloned()
        .collect::<HashSet<String>>();
    let encoded_reverse = encoded
        .clone()
        .into_iter()
        .map(|(input_file_name, output_file_name)| (output_file_name, input_file_name))
        .collect::<HashMap<String, String>>();

    // Songs removed from the destination directory but encoded previously
    for removed_file in removed_output {
        if let Some(file_to_recode) = encoded_reverse.get(&removed_file) {
            encoded.remove(file_to_recode);
        }
    }

    // Songs removed from the encoded.json but present in the output dir with the correct name
    let encoded_not_saved_output: HashSet<String> =
        output.difference(&encoded_output).cloned().collect();
    for input_file_name in input.clone() {
        let output_file_name = create_output_file_name(input_file_name.clone(), &config);
        if encoded_not_saved_output.contains(&output_file_name) {
            encoded.insert(input_file_name, output_file_name);
        }
    }

    // Songs encoded with the wrong extension
    for (input_file, output_file) in encoded.clone() {
        let input_file_extension = Path::new(&input_file)
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        if (&config)
            .extensions_to_encode
            .contains(&input_file_extension)
        {
            let output_file_extension = Path::new(&output_file)
                .extension()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            if output_file_extension != (&config).encoded_extension {
                encoded.remove(&input_file);
            }
        }
    }

    // Encoded songs with an incorrect name (After a config change) - rename without re-encoding
    let mut output_to_rename = HashMap::new();
    for input_file in encoded.keys() {
        let new_name = create_output_file_name(input_file.to_string(), &config);
        let old_name = encoded.get(input_file).unwrap().to_string();
        if new_name != old_name {
            output_to_rename.insert(old_name, new_name);
        }
    }

    // All input songs which are not present in encoded need to be processed
    // All output songs which are not present in encoded need to be deleted
    let encoded_input: HashSet<String> = encoded.keys().cloned().collect();
    let encoded_output: HashSet<String> = encoded.values().cloned().collect();
    let input_to_process: HashSet<String> = input.difference(&encoded_input).cloned().collect();
    let output_to_delete: HashSet<String> = output.difference(&encoded_output).cloned().collect();

    // Ask user whether to continue
    println!(
        "{}",
        bold_green.apply_to(format!(
            "{} songs to encode/copy, {} to rename and {} to delete",
            input_to_process.len(),
            output_to_rename.len(),
            output_to_delete.len()
        ))
    );

    if !args.yes {
        if !Confirm::new()
            .with_prompt("Do you want to continue?")
            .interact()?
        {
            println!("Aborting");
            std::process::exit(3);
        }
    }

    // Process all files

    // Delete files
    for file_to_delete in output_to_delete {
        println!("Deleting {}", file_to_delete);
        if args.dry_run {
            eprintln!("Skipping delete as --dry-run is set");
        } else {
            fs::remove_file(Path::new(&config.output_directory).join(file_to_delete))?;
        }
    }

    // Rename already encoded
    for (old_file_name, new_file_name) in output_to_rename {
        println!("Renaming {} to {}", old_file_name, new_file_name);
        if args.dry_run {
            eprintln!("Skipping rename as --dry-run is set");
        } else {
            fs::rename(
                Path::new(&config.output_directory).join(old_file_name),
                Path::new(&config.output_directory).join(new_file_name),
            )?;
        }
    }

    // Encode or copy
    for input_file_name in input_to_process {
        let file_extension = Path::new(&input_file_name)
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let output_file_name = create_output_file_name(input_file_name.clone(), &config);
        if (&config).extensions_to_encode.contains(&file_extension) {
            println!(
                "Encoding {} to {} with ffmpeg params {}",
                input_file_name, output_file_name, config.ffmpeg_params
            );
            if args.dry_run {
                eprintln!("Skipping encode as --dry-run is set");
            } else {
                let input_file_path = Path::new(&config.input_directory).join(input_file_name);
                let output_file_path = Path::new(&config.output_directory).join(output_file_name);
                let mut params = vec!["-i", input_file_path.to_str().unwrap()];
                let mut config_params: Vec<&str> = (&config.ffmpeg_params).split(" ").collect();
                params.append(&mut config_params);
                params.push(output_file_path.to_str().unwrap());
                let mut command = Command::new("ffmpeg");
                command.args(params);
                if args.quiet {
                    command.output().expect("Failed to execute ffmpeg");
                } else {
                    command.status().expect("Failed to execute ffmpeg");
                }
                if config.copy_covers == Some(true) {
                    println!("Copying audio cover");
                    copy_pictures(input_file_path, output_file_path)?;
                }
            }
        } else {
            println!("Copying {} to {}", input_file_name, output_file_name);
            if args.dry_run {
                eprintln!("Skipping copy as --dry-run is set");
            } else {
                fs::copy(
                    Path::new(&config.input_directory).join(input_file_name),
                    Path::new(&config.output_directory).join(output_file_name),
                )?;
            }
        }
    }

    // Save info about processed files to a JSON
    println!("{}", bold_green.apply_to("Done processing files"));
    if args.dry_run {
        eprintln!("Skipping save to JSON as --dry-run is set");
    } else {
        let encoded = create_final_encoded_map(input, &config);
        let encoded_file = File::create(args.encoded)?;
        let encoded_file_writer = BufWriter::new(encoded_file);
        serde_json::to_writer(encoded_file_writer, &encoded)?;
    }

    Ok(())
}

fn create_final_encoded_map(input: HashSet<String>, config: &Config) -> HashMap<String, String> {
    input
        .into_iter()
        .map(|input_file_name| {
            (
                input_file_name.clone(),
                create_output_file_name(input_file_name, &config),
            )
        })
        .collect()
}

fn create_output_file_name(input_file_name: String, config: &Config) -> String {
    let input_file_extension = Path::new(&input_file_name)
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let input_file_stem = Path::new(&input_file_name)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let mut new_file_name = input_file_name;
    if config.extensions_to_encode.contains(&input_file_extension) {
        new_file_name = format!("{}.{}", input_file_stem, &config.encoded_extension);
    }
    if config.remove_round_brackets == Some(true) {
        lazy_static! {
            static ref REGEX_SPACE_FIRST: Regex = Regex::new(r" \(.*?\)").unwrap();
            static ref REGEX_SPACE_LAST: Regex = Regex::new(r"\(.*?\) ").unwrap();
            static ref REGEX: Regex = Regex::new(r"\(.*?\)").unwrap();
        }
        new_file_name = REGEX_SPACE_FIRST
            .replace_all(&new_file_name, "")
            .to_string();
        new_file_name = REGEX_SPACE_LAST.replace_all(&new_file_name, "").to_string();
        new_file_name = REGEX.replace_all(&new_file_name, "").to_string();
    }
    if config.remove_square_brackets == Some(true) {
        lazy_static! {
            static ref REGEX_SPACE_FIRST: Regex = Regex::new(r" \[.*?\]").unwrap();
            static ref REGEX_SPACE_LAST: Regex = Regex::new(r"\[.*?\] ").unwrap();
            static ref REGEX: Regex = Regex::new(r"\[.*?\]").unwrap();
        }
        new_file_name = REGEX_SPACE_FIRST
            .replace_all(&new_file_name, "")
            .to_string();
        new_file_name = REGEX_SPACE_LAST.replace_all(&new_file_name, "").to_string();
        new_file_name = REGEX.replace_all(&new_file_name, "").to_string();
    }
    if config.remove_curly_brackets == Some(true) {
        lazy_static! {
            static ref REGEX_SPACE_FIRST: Regex = Regex::new(r" \{.*?\}").unwrap();
            static ref REGEX_SPACE_LAST: Regex = Regex::new(r"\{.*?\} ").unwrap();
            static ref REGEX: Regex = Regex::new(r"\{.*?\}").unwrap();
        }
        new_file_name = REGEX_SPACE_FIRST
            .replace_all(&new_file_name, "")
            .to_string();
        new_file_name = REGEX_SPACE_LAST.replace_all(&new_file_name, "").to_string();
        new_file_name = REGEX.replace_all(&new_file_name, "").to_string();
    }
    if config.remove_angle_brackets == Some(true) {
        lazy_static! {
            static ref REGEX_SPACE_FIRST: Regex = Regex::new(r" <.*?>").unwrap();
            static ref REGEX_SPACE_LAST: Regex = Regex::new(r"<.*?> ").unwrap();
            static ref REGEX: Regex = Regex::new(r"<.*?>").unwrap();
        }
        new_file_name = REGEX_SPACE_FIRST
            .replace_all(&new_file_name, "")
            .to_string();
        new_file_name = REGEX_SPACE_LAST.replace_all(&new_file_name, "").to_string();
        new_file_name = REGEX.replace_all(&new_file_name, "").to_string();
    }
    new_file_name
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Config {
    pub input_directory: String,
    pub output_directory: String,
    pub extensions_to_encode: Vec<String>,
    pub encoded_extension: String,
    pub copy_covers: Option<bool>,
    pub ffmpeg_params: String,
    pub remove_round_brackets: Option<bool>,
    pub remove_square_brackets: Option<bool>,
    pub remove_curly_brackets: Option<bool>,
    pub remove_angle_brackets: Option<bool>,
}
