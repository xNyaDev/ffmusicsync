use std::path::PathBuf;

use lofty::{Probe, TagExt};

pub fn copy_pictures(input: PathBuf, output: PathBuf) -> lofty::error::Result<()> {
    let input_tagged_file = Probe::open(input)?.guess_file_type()?.read(false)?;
    let input_tag = input_tagged_file.primary_tag().unwrap();
    let mut output_tagged_file = Probe::open(&output)?.guess_file_type()?.read(true)?;
    let output_tag = output_tagged_file.primary_tag_mut().unwrap();
    input_tag.pictures().into_iter().cloned().for_each(
        |picture| {
            output_tag.push_picture(picture);
        }
    );
    output_tag.save_to_path(output)?;
    Ok(())
}