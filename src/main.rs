mod calculation;
mod parser;

use crate::parser::*;
use std::env;

#[derive(Debug)]
pub enum EnumError {
    NoInputFile,
    InvalidCSV,
    InvalidType,
    CannotConvert(String),
    CannotWriteCsv,
    CannotWriteLine,
    CannotOpenCsv,
    FileNotPresent,
}

fn main() -> Result<(), EnumError> {
    let file_input = env::args()
        .into_iter()
        .nth(1)
        .ok_or(EnumError::NoInputFile)?;

    let csv_parsed = parser::read_csv(file_input)?;

    let calculated_positions = calculation::calculate_position_for_each_client(csv_parsed);

    parser::write_out_positions(calculated_positions.into_iter().map(Into::into).collect());

    Ok(())
}
