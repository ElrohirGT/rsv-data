//!Core module used for encoding and decoding RSV data
//!
//!Here's an example for input to the encoder:
//!
//!```
//!let input: Vec<Vec<Option<String>>> = vec![
//!    vec![Some("Hello user!".to_string()), None],
//!    vec![Some("\n\\\'\"".to_string()), Some("😁🔃📖".to_string())]
//!];
//!```

use thiserror::Error;

pub const VALUE_TERMINATOR: u8 = 0xFF;
pub const ROW_TERMINATOR: u8 = 0xFD;
pub const NULL_VALUE: u8 = 0xFE;

pub fn encode_rsv<T: ToString>(rows: &[Vec<Option<T>>]) -> Vec<u8> {
    rows.iter().fold(vec![], |mut result, row| {
        let mut row_bytes = row
            .iter()
            .map(|v| match v {
                Some(t_value) => t_value.to_string().into_bytes(),
                None => vec![NULL_VALUE],
            })
            .fold(vec![], |mut row_result, mut value_in_bytes| {
                row_result.append(&mut value_in_bytes);
                row_result.push(VALUE_TERMINATOR);
                row_result
            });
        result.append(&mut row_bytes);
        result.push(ROW_TERMINATOR);
        result
    })
}

#[derive(Debug, Error)]
pub enum DecodeRSVErrors {
    #[error("The RSV file ends unexpectedly!")]
    IncompleteRSVDocument,
    #[error("The RSV row on byte number `{0}` ends unexpectedly!")]
    IncompleteRSVRow(usize),
    #[error("Invalid UTF-8 byte sequence: {0:?}!")]
    InvalidStringValue(#[from] std::string::FromUtf8Error),
}

pub fn decode_rsv(bytes: &[u8]) -> Result<Vec<Vec<Option<String>>>, DecodeRSVErrors> {
    if bytes.last() != Some(&ROW_TERMINATOR) {
        Err(DecodeRSVErrors::IncompleteRSVDocument)?
    }

    let mut result: Vec<Vec<Option<String>>> = vec![];
    let mut current_row: Vec<Option<String>> = vec![];
    let mut value_start_index = 0;

    for i in 0..bytes.len() {
        match bytes[i] {
            VALUE_TERMINATOR => {
                let length = i - value_start_index;

                match (length, bytes[value_start_index]) {
                    (0, _) => current_row.push(Some(String::new())),
                    (1, NULL_VALUE) => current_row.push(None),
                    (_, _) => {
                        let value_bytes = bytes[value_start_index..i].to_vec();
                        match String::from_utf8(value_bytes) {
                            Ok(str_value) => current_row.push(Some(str_value)),
                            Err(err) => Err(DecodeRSVErrors::InvalidStringValue(err))?,
                        }
                    }
                }

                value_start_index = i + 1;
            }
            ROW_TERMINATOR => {
                if i > 0 && value_start_index != i {
                    Err(DecodeRSVErrors::IncompleteRSVRow(i + 1))?
                }

                result.push(current_row);
                current_row = Vec::new();
                value_start_index = i + 1;
            }
            _ => {}
        }
    }

    Ok(result)
}
