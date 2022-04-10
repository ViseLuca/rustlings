use crate::calculation::{get_transaction_amount, has_a_dispute_not_closed, Position};
use crate::{CSVParsed, EnumType};
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;

#[test]
fn has_a_dispute_on_a_transaction() {
    let transaction = &CSVParsed {
        r#type: EnumType::Chargeback,
        client: 1,
        transaction_id: 2,
        amount: None,
    };

    let transactions = &vec![
        CSVParsed {
            r#type: EnumType::Deposit,
            client: 1,
            transaction_id: 1,
            amount: Some(Decimal::new(10, 2)),
        },
        CSVParsed {
            r#type: EnumType::Withdrawal,
            client: 1,
            transaction_id: 2,
            amount: Some(Decimal::new(3, 2)),
        },
        CSVParsed {
            r#type: EnumType::Withdrawal,
            client: 1,
            transaction_id: 3,
            amount: Some(Decimal::new(3, 2)),
        },
        CSVParsed {
            r#type: EnumType::Dispute,
            client: 1,
            transaction_id: 2,
            amount: None,
        },
    ];

    assert_eq!(true, has_a_dispute_not_closed(transaction, transactions),);

    let transaction = &CSVParsed {
        r#type: EnumType::Chargeback,
        client: 1,
        transaction_id: 1,
        amount: None,
    };

    assert_eq!(false, has_a_dispute_not_closed(transaction, transactions),);
}

#[test]
fn transaction_amount() {
    let transaction = &CSVParsed {
        r#type: EnumType::Chargeback,
        client: 1,
        transaction_id: 2,
        amount: None,
    };

    let transactions = &vec![
        CSVParsed {
            r#type: EnumType::Deposit,
            client: 1,
            transaction_id: 1,
            amount: Some(Decimal::new(10, 2)),
        },
        CSVParsed {
            r#type: EnumType::Withdrawal,
            client: 1,
            transaction_id: 2,
            amount: Some(Decimal::new(3, 2)),
        },
        CSVParsed {
            r#type: EnumType::Withdrawal,
            client: 1,
            transaction_id: 3,
            amount: Some(Decimal::new(3, 2)),
        },
        CSVParsed {
            r#type: EnumType::Dispute,
            client: 1,
            transaction_id: 2,
            amount: None,
        },
    ];

    assert_eq!(
        Some(Decimal::new(3, 2)),
        get_transaction_amount(transaction, transactions)
    );

    let transaction = &CSVParsed {
        r#type: EnumType::Chargeback,
        client: 1,
        transaction_id: 5,
        amount: None,
    };

    assert_eq!(None, get_transaction_amount(transaction, transactions));
}

#[test]
fn no_position_changed_with_only_chargeback() {
    let pos = Position::new(1);
    let transaction = CSVParsed {
        r#type: EnumType::Chargeback,
        client: 1,
        transaction_id: 5,
        amount: None,
    };

    let transactions = &vec![CSVParsed {
        r#type: EnumType::Chargeback,
        client: 1,
        transaction_id: 5,
        amount: None,
    }];

    let pos = pos.manage_transaction(&transaction, transactions);

    let result = Position {
        client: 1,
        available: Decimal::zero(),
        held: Decimal::zero(),
        total: Decimal::zero(),
        locked: false,
    };

    assert_eq!(result, pos);
}

#[test]
fn no_position_changed_with_only_withdrawal() {
    let pos = Position::new(1);
    let transaction = CSVParsed {
        r#type: EnumType::Withdrawal,
        client: 1,
        transaction_id: 5,
        amount: Some(Decimal::new(10, 2)),
    };

    let transactions = &vec![CSVParsed {
        r#type: EnumType::Withdrawal,
        client: 1,
        transaction_id: 5,
        amount: Some(Decimal::new(10, 2)),
    }];

    let pos = pos.manage_transaction(&transaction, transactions);

    let result = Position {
        client: 1,
        available: Decimal::zero(),
        held: Decimal::zero(),
        total: Decimal::zero(),
        locked: false,
    };

    assert_eq!(result, pos);
}

#[test]
fn position_changed_with_deposit() {
    let pos = Position::new(1);
    let transaction = CSVParsed {
        r#type: EnumType::Deposit,
        client: 1,
        transaction_id: 5,
        amount: Some(Decimal::new(10, 2)),
    };

    let transactions = &vec![CSVParsed {
        r#type: EnumType::Deposit,
        client: 1,
        transaction_id: 5,
        amount: Some(Decimal::new(10, 2)),
    }];

    let pos = pos.manage_transaction(&transaction, transactions);

    let result = Position {
        client: 1,
        available: Decimal::new(10, 2),
        held: Decimal::zero(),
        total: Decimal::new(10, 2),
        locked: false,
    };

    assert_eq!(result, pos);
}

#[test]
fn position_changed_with_deposit_and_a_partial_withdrawal() {
    let pos = Position::new(1);

    let transactions = &vec![
        CSVParsed {
            r#type: EnumType::Deposit,
            client: 1,
            transaction_id: 1,
            amount: Some(Decimal::new(10, 2)),
        },
        CSVParsed {
            r#type: EnumType::Withdrawal,
            client: 1,
            transaction_id: 2,
            amount: Some(Decimal::new(5, 2)),
        },
    ];

    let transaction = CSVParsed {
        r#type: EnumType::Deposit,
        client: 1,
        transaction_id: 1,
        amount: Some(Decimal::new(10, 2)),
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    let transaction = CSVParsed {
        r#type: EnumType::Withdrawal,
        client: 1,
        transaction_id: 2,
        amount: Some(Decimal::new(5, 2)),
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    let result = Position {
        client: 1,
        available: Decimal::new(5, 2),
        held: Decimal::zero(),
        total: Decimal::new(5, 2),
        locked: false,
    };

    assert_eq!(result, pos);
}

#[test]
fn position_not_changed_with_deposit_and_a_withdrawal_more_than_available() {
    let pos = Position::new(1);

    let transactions = &vec![
        CSVParsed {
            r#type: EnumType::Deposit,
            client: 1,
            transaction_id: 1,
            amount: Some(Decimal::new(10, 2)),
        },
        CSVParsed {
            r#type: EnumType::Withdrawal,
            client: 1,
            transaction_id: 2,
            amount: Some(Decimal::new(15, 2)),
        },
    ];

    let transaction = CSVParsed {
        r#type: EnumType::Deposit,
        client: 1,
        transaction_id: 1,
        amount: Some(Decimal::new(10, 2)),
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    let transaction = CSVParsed {
        r#type: EnumType::Withdrawal,
        client: 1,
        transaction_id: 2,
        amount: Some(Decimal::new(15, 2)),
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    let result = Position {
        client: 1,
        available: Decimal::new(10, 2),
        held: Decimal::zero(),
        total: Decimal::new(10, 2),
        locked: false,
    };

    assert_eq!(result, pos);
}

#[test]
fn position_locked_after_a_chargeback_on_dispute() {
    let pos = Position::new(1);

    let transactions = &vec![
        CSVParsed {
            r#type: EnumType::Deposit,
            client: 1,
            transaction_id: 1,
            amount: Some(Decimal::new(10, 2)),
        },
        CSVParsed {
            r#type: EnumType::Dispute,
            client: 1,
            transaction_id: 1,
            amount: None,
        },
        CSVParsed {
            r#type: EnumType::Chargeback,
            client: 1,
            transaction_id: 1,
            amount: None,
        },
    ];

    let transaction = CSVParsed {
        r#type: EnumType::Deposit,
        client: 1,
        transaction_id: 1,
        amount: Some(Decimal::new(10, 2)),
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    let transaction = CSVParsed {
        r#type: EnumType::Dispute,
        client: 1,
        transaction_id: 1,
        amount: None,
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    let transaction = CSVParsed {
        r#type: EnumType::Chargeback,
        client: 1,
        transaction_id: 1,
        amount: None,
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    let result = Position {
        client: 1,
        available: Decimal::new(0, 2),
        held: Decimal::zero(),
        total: Decimal::new(0, 2),
        locked: true,
    };

    assert_eq!(result, pos);
}

#[test]
fn position_not_locked_after_a_chargeback_on_resolved_dispute() {
    let pos = Position::new(1);

    let transactions = &vec![
        CSVParsed {
            r#type: EnumType::Deposit,
            client: 1,
            transaction_id: 1,
            amount: Some(Decimal::new(10, 2)),
        },
        CSVParsed {
            r#type: EnumType::Dispute,
            client: 1,
            transaction_id: 1,
            amount: None,
        },
        CSVParsed {
            r#type: EnumType::Resolve,
            client: 1,
            transaction_id: 1,
            amount: None,
        },
        CSVParsed {
            r#type: EnumType::Chargeback,
            client: 1,
            transaction_id: 1,
            amount: None,
        },
    ];

    let transaction = CSVParsed {
        r#type: EnumType::Deposit,
        client: 1,
        transaction_id: 1,
        amount: Some(Decimal::new(10, 2)),
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    let result = Position {
        client: 1,
        available: Decimal::new(10, 2),
        held: Decimal::zero(),
        total: Decimal::new(10, 2),
        locked: false,
    };

    assert_eq!(result, pos);

    let transaction = CSVParsed {
        r#type: EnumType::Dispute,
        client: 1,
        transaction_id: 1,
        amount: None,
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    let result = Position {
        client: 1,
        available: Decimal::zero(),
        held: Decimal::new(10, 2),
        total: Decimal::new(10, 2),
        locked: false,
    };

    assert_eq!(result, pos);

    let transaction = CSVParsed {
        r#type: EnumType::Resolve,
        client: 1,
        transaction_id: 1,
        amount: None,
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    let result = Position {
        client: 1,
        available: Decimal::new(10, 2),
        held: Decimal::zero(),
        total: Decimal::new(10, 2),
        locked: false,
    };

    assert_eq!(result, pos);

    let transaction = CSVParsed {
        r#type: EnumType::Chargeback,
        client: 1,
        transaction_id: 1,
        amount: None,
    };

    let pos = pos.manage_transaction(&transaction, transactions);

    assert_eq!(result, pos);
}
