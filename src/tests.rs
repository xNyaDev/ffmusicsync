#[cfg(test)]
use super::*;

#[test]
fn destination_file_names() {
    let config = Config {
        input_directory: "".to_string(),
        output_directory: "".to_string(),
        extensions_to_encode: vec!["flac".to_string()],
        encoded_extension: "ogg".to_string(),
        ffmpeg_params: "".to_string(),
        remove_round_brackets: Some(false),
        remove_square_brackets: Some(true),
        remove_curly_brackets: None,
        remove_angle_brackets: None,
    };

    let input = "Test - Song (Original Mix) [2022] <Test> {}.flac".to_string();
    assert_eq!(
        "Test - Song (Original Mix) <Test> {}.ogg".to_string(),
        create_output_file_name(input, &config)
    );

    let input = "[Multi Test] Test - [] Song [2022].ogg".to_string();
    assert_eq!(
        "Test - Song.ogg".to_string(),
        create_output_file_name(input, &config)
    );

    let config = Config {
        input_directory: "".to_string(),
        output_directory: "".to_string(),
        extensions_to_encode: vec!["flac".to_string()],
        encoded_extension: "ogg".to_string(),
        ffmpeg_params: "".to_string(),
        remove_round_brackets: Some(true),
        remove_square_brackets: Some(true),
        remove_curly_brackets: Some(true),
        remove_angle_brackets: Some(true),
    };

    let input = "Test - Song (Original Mix) [2022] <Test> {}.mp3".to_string();
    assert_eq!(
        "Test - Song.mp3".to_string(),
        create_output_file_name(input, &config)
    );
}
