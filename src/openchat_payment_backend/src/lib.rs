use candid::{CandidType, Deserialize};
use ic_cdk::api::caller;
use std::collections::HashMap;
use ic_cdk::storage;
use std::cell::RefCell;

thread_local! {
    static LEDGER: RefCell<HashMap<String, Wallet>> = RefCell::new(HashMap::new());
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Wallet {
    balance: u64,
    transaction_history: Vec<Transaction>
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Transaction {
    from: String,
    to: String,
    amount: u64,
    timestamp: u64,
}

#[ic_cdk::update]
fn create_wallet(username: String) -> Result<(), String> {
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        if ledger.contains_key(&username) {
            return Err("Wallet already exists".to_string());
        }
        ledger.insert(username, Wallet {
            balance: 1000, // Initial balance
            transaction_history: Vec::new()
        });
        Ok(())
    })
}

#[ic_cdk::update]
fn send_payment(to: String, amount: u64) -> Result<(), String> {
    let sender = caller().to_string();
    
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        
        // Check sender balance
        let sender_balance = ledger.get(&sender)
            .map(|w| w.balance)
            .ok_or("Sender wallet not found")?;
            
        if sender_balance < amount {
            return Err("Insufficient funds".to_string());
        }

        let transaction = Transaction {
            from: sender.clone(),
            to: to.clone(),
            amount,
            timestamp: ic_cdk::api::time(),
        };

        // Update receiver
        ledger.entry(to)
            .and_modify(|w| {
                w.balance += amount;
                w.transaction_history.push(transaction.clone());
            })
            .or_insert(Wallet {
                balance: amount,
                transaction_history: vec![transaction.clone()]
            });

        // Update sender
        ledger.entry(sender)
            .and_modify(|w| {
                w.balance -= amount;
                w.transaction_history.push(transaction);
            });

        Ok(())
    })
}

#[ic_cdk::query]
fn get_balance(username: String) -> Result<u64, String> {
    LEDGER.with(|ledger| {
        let ledger = ledger.borrow();
        ledger.get(&username)
            .map(|wallet| wallet.balance)
            .ok_or("Wallet not found".to_string())
    })
}

#[ic_cdk::query]
fn get_transaction_history(username: String) -> Result<Vec<Transaction>, String> {
    LEDGER.with(|ledger| {
        let ledger = ledger.borrow();
        ledger.get(&username)
            .map(|wallet| wallet.transaction_history.clone())
            .ok_or("Wallet not found".to_string())
    })
}

#[ic_cdk::init]
fn init() {
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        ledger.clear();
    });
}

ic_cdk::export_candid!();
