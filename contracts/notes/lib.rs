#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Address, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Loan {
    pub id: u64,
    pub borrower: Address,
    pub amount: i128,
    pub interest_bps: u32,
    pub is_repaid: bool,
}

const LOAN_DATA: Symbol = symbol_short!("LOAN_DATA");
const POOL_BAL: Symbol = symbol_short!("POOL_BAL");

#[contract]
pub struct LendingContract;

#[contractimpl]
impl LendingContract {

    pub fn get_loans(env: Env) -> Vec<Loan> {
        env.storage().instance().get(&LOAN_DATA).unwrap_or(Vec::new(&env))
    }

    pub fn get_pool_balance(env: Env) -> i128 {
        env.storage().instance().get(&POOL_BAL).unwrap_or(0i128)
    }

    pub fn deposit(env: Env, amount: i128) -> i128 {
        assert!(amount > 0, "Deposit amount must be positive");
        let current: i128 = env.storage().instance().get(&POOL_BAL).unwrap_or(0i128);
        let new_balance = current + amount;
        env.storage().instance().set(&POOL_BAL, &new_balance);
        new_balance
    }

    pub fn borrow(env: Env, borrower: Address, amount: i128, interest_bps: u32) -> u64 {
        borrower.require_auth();
        assert!(amount > 0, "Borrow amount must be positive");
        assert!(interest_bps <= 10_000, "Interest rate cannot exceed 100%");
        let pool: i128 = env.storage().instance().get(&POOL_BAL).unwrap_or(0i128);
        assert!(pool >= amount, "Insufficient pool liquidity");
        env.storage().instance().set(&POOL_BAL, &(pool - amount));
        let loan_id = env.prng().gen_range(1u64..u64::MAX);
        let loan = Loan {
            id: loan_id,
            borrower,
            amount,
            interest_bps,
            is_repaid: false,
        };
        let mut loans: Vec<Loan> = env.storage().instance().get(&LOAN_DATA).unwrap_or(Vec::new(&env));
        loans.push_back(loan);
        env.storage().instance().set(&LOAN_DATA, &loans);
        loan_id
    }

    pub fn repay(env: Env, borrower: Address, loan_id: u64) -> i128 {
        borrower.require_auth();
        let mut loans: Vec<Loan> = env.storage().instance().get(&LOAN_DATA).unwrap_or(Vec::new(&env));
        for i in 0..loans.len() {
            let loan = loans.get(i).unwrap();
            if loan.id == loan_id {
                assert!(!loan.is_repaid, "Loan already repaid");
                assert!(loan.borrower == borrower, "Not your loan");
                let interest = (loan.amount * loan.interest_bps as i128) / 10_000;
                let repay_amount = loan.amount + interest;
                let repaid_loan = Loan {
                    id: loan.id,
                    borrower: loan.borrower,
                    amount: loan.amount,
                    interest_bps: loan.interest_bps,
                    is_repaid: true,
                };
                loans.set(i, repaid_loan);
                env.storage().instance().set(&LOAN_DATA, &loans);
                let pool: i128 = env.storage().instance().get(&POOL_BAL).unwrap_or(0i128);
                env.storage().instance().set(&POOL_BAL, &(pool + repay_amount));
                return repay_amount;
            }
        }
        panic!("Loan not found");
    }
}