use crate::calculation::Position;
use crate::EnumError;
use csv::Trim;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs;
use std::str::FromStr;

#[derive(Deserialize)]
struct CSVStruct {
    r#type: String,
    client: String,
    tx: String,
    amount: String,
}

#[derive(Debug, Clone)]
pub struct CSVParsed {
    pub r#type: EnumType,
    pub client: u16,
    pub transaction_id: u32,
    pub amount: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TryFrom<String> for EnumType {
    type Error = EnumError;

    fn try_from(r#type: String) -> Result<Self, Self::Error> {
        match r#type.as_str() {
            "deposit" => Ok(Self::Deposit),
            "withdrawal" => Ok(Self::Withdrawal),
            "dispute" => Ok(Self::Dispute),
            "resolve" => Ok(Self::Resolve),
            "chargeback" => Ok(Self::Chargeback),
            _ => Err(EnumError::InvalidType),
        }
    }
}

impl TryFrom<CSVStruct> for CSVParsed {
    type Error = EnumError;

    fn try_from(csv_struct: CSVStruct) -> Result<Self, EnumError> {
        Ok(Self {
            r#type: csv_struct.r#type.try_into()?,
            client: csv_struct
                .client
                .trim()
                .parse::<u16>()
                .map_err(|_| EnumError::CannotConvert("client".to_string()))?,
            transaction_id: csv_struct
                .tx
                .trim()
                .parse::<u32>()
                .map_err(|_| EnumError::CannotConvert("transaction_id".to_string()))?,
            amount: if csv_struct.amount.trim().is_empty() {
                None
            } else {
                Some(
                    Decimal::from_str(csv_struct.amount.trim())
                        .map_err(|_| EnumError::CannotConvert("amount".to_string()))?,
                )
            },
        })
    }
}

pub fn read_csv(file_name: String) -> Result<Vec<CSVParsed>, EnumError> {
    let contents = fs::read_to_string(file_name).map_err(|_| EnumError::FileNotPresent)?;

    let data: &[u8] = &contents.into_bytes();
    let mut reader = csv::ReaderBuilder::new().trim(Trim::All).from_reader(data);

    let csv_result: Result<Vec<CSVStruct>, csv::Error> = reader.deserialize().collect(); // Fail at the first error
    let csv = csv_result.map_err(|err| {
        println!("{:?}", err);
        EnumError::InvalidCSV
    })?;

    csv.into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<CSVParsed>, EnumError>>()
}

#[derive(Serialize)]
pub struct CSVOutput {
    client: String,
    available: String,
    held: String,
    total: String,
    locked: String,
}

impl ToString for CSVOutput {
    fn to_string(&self) -> String {
        self.client.to_string()
            + ","
            + self.available.as_str()
            + ","
            + self.held.as_str()
            + ","
            + self.total.as_str()
            + ","
            + self.locked.as_str()
    }
}

impl From<Position> for CSVOutput {
    fn from(position: Position) -> Self {
        Self {
            client: position.client.to_string(),
            available: position.available.to_string(),
            held: position.held.to_string(),
            total: position.total.to_string(),
            locked: position.locked.to_string(),
        }
    }
}

pub fn write_out_positions(positions: Vec<CSVOutput>) {
    println!("client,available,held,total,locked");
    for position in positions {
        println!("{}", position.to_string())
    }
}
