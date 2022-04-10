use crate::{CSVParsed, EnumType};
use rust_decimal::Decimal;
use std::collections::HashSet;
use std::ops::{Add, Sub};

#[cfg(test)]
#[path = "test/calculation_test.rs"]
mod calculation_test;

#[derive(Debug, PartialEq)]
pub struct Position {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Position {
    pub fn new(client_id: u16) -> Position {
        Self {
            client: client_id,
            available: Default::default(),
            held: Default::default(),
            total: Default::default(),
            locked: Default::default(),
        }
    }

    pub fn manage_transaction(
        self,
        transaction: &CSVParsed,
        client_transactions: &[CSVParsed],
    ) -> Position {
        match (&transaction.r#type, self.locked) {
            (_, true) => self.none(),
            (EnumType::Deposit, _) => self.deposit(transaction),
            (EnumType::Withdrawal, _) => self.withdrawal(transaction),
            (EnumType::Dispute, _) => self.dispute(transaction, client_transactions),
            (EnumType::Resolve, _) => self.resolve(transaction, client_transactions),
            (EnumType::Chargeback, _) => self.chargeback(transaction, client_transactions),
        }
    }

    fn deposit(self, transaction: &CSVParsed) -> Position {
        Position {
            available: transaction.amount.map_or(self.available, |amount| {
                Decimal::add(self.available, amount)
            }),
            total: transaction
                .amount
                .map_or(self.total, |amount| Decimal::add(self.total, amount)),
            ..self
        }
    }

    fn withdrawal(self, transaction: &CSVParsed) -> Position {
        if transaction
            .amount
            .map_or(false, |amount| amount < self.available)
        {
            Position {
                available: transaction.amount.map_or(self.available, |amount| {
                    Decimal::sub(self.available, amount)
                }),
                total: transaction
                    .amount
                    .map_or(self.total, |amount| Decimal::sub(self.total, amount)),
                ..self
            }
        } else {
            self
        }
    }

    fn dispute(self, transaction: &CSVParsed, client_transactions: &[CSVParsed]) -> Position {
        if let Some(value) = get_transaction_amount(transaction, client_transactions) {
            Position {
                available: Decimal::sub(self.available, value),
                held: Decimal::add(self.held, value),
                ..self
            }
        } else {
            self
        }
    }

    fn resolve(self, transaction: &CSVParsed, client_transactions: &[CSVParsed]) -> Position {
        match (
            has_a_dispute_transaction(transaction, client_transactions),
            get_transaction_amount(transaction, client_transactions),
        ) {
            (true, Some(value)) => Position {
                available: Decimal::add(self.available, value),
                held: Decimal::sub(self.held, value),
                ..self
            },
            _ => self,
        }
    }

    fn chargeback(self, transaction: &CSVParsed, client_transactions: &[CSVParsed]) -> Position {
        match (
            has_a_dispute_not_closed(transaction, client_transactions),
            get_transaction_amount(transaction, client_transactions),
        ) {
            (true, Some(value)) => Position {
                held: Decimal::sub(self.held, value),
                total: Decimal::sub(self.total, value),
                locked: true,
                ..self
            },
            _ => self,
        }
    }

    fn none(self) -> Position {
        self
    }
}

fn get_transaction_amount(
    transaction: &CSVParsed,
    client_transactions: &[CSVParsed],
) -> Option<Decimal> {
    client_transactions
        .iter()
        .filter(|t| t.transaction_id == transaction.transaction_id && t.amount.is_some())
        .last()
        .and_then(|t| t.amount)
}

fn has_a_dispute_not_closed(transaction: &CSVParsed, client_transactions: &[CSVParsed]) -> bool {
    has_a_dispute_transaction(transaction, client_transactions)
        && client_transactions
            .iter()
            .filter(|t| {
                t.transaction_id == transaction.transaction_id && t.r#type == EnumType::Resolve
            })
            .count()
            != 1
}

fn has_a_dispute_transaction(transaction: &CSVParsed, client_transactions: &[CSVParsed]) -> bool {
    client_transactions
        .iter()
        .filter(|t| t.transaction_id == transaction.transaction_id && t.r#type == EnumType::Dispute)
        .count()
        == 1
}

pub fn calculate_position_for_each_client(positions: Vec<CSVParsed>) -> Vec<Position> {
    let clients: HashSet<u16> = positions.iter().map(|pos| pos.client).collect();

    let mut client_positions: Vec<Position> = vec![];

    for client in clients {
        let client_transactions: Vec<CSVParsed> = positions
            .clone()
            .into_iter()
            .filter(|pos| pos.client == client)
            .collect();
        let position: Position = client_transactions
            .iter()
            .fold(Position::new(client), |position, transaction| {
                position.manage_transaction(transaction, &client_transactions)
            });

        client_positions.push(position);
    }

    client_positions
}
