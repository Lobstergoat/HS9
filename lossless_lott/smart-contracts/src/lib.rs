use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise, U128};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::convert::TryInto;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub staking_contract: AccountId,
    pub investors: UnorderedMap<AccountId, Balance>,
    pub players: UnorderedMap<AccountId, Balance>,
    pub total_stake: Balance,
    pub min_deposit: Balance,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, staking_contract: AccountId, min_deposit: U128) -> Self {
        Self {
            owner_id,
            staking_contract,
            investors: UnorderedMap::new(b"i"),
            players: UnorderedMap::new(b"p"),
            total_stake: 0,
            min_deposit: min_deposit.0,
        }
    }

    pub fn invest(&mut self, amount: U128) {
        let investor = env::predecessor_account_id();
        let investment_amount = amount.0;
        
        self.investors.insert(&investor, &investment_amount);
        self.total_stake += investment_amount;
        
        Promise::new(self.staking_contract.clone())
            .function_call("stake".to_string(), amount.into(), 0, env::prepaid_gas() / 2);
    }

    pub fn play(&mut self, amount: U128) {
        let player = env::predecessor_account_id();
        let play_amount = amount.0;
        assert!(play_amount >= self.min_deposit, "Deposit too low");
        
        self.players.insert(&player, &play_amount);
    }

    pub fn total_investment(&self) -> Balance {
        self.investors.values_as_vector().iter().sum()
    }

    fn total_player_deposit(&self) -> Balance {
        self.players.values_as_vector().iter().sum()
    }

    fn simulate_profit(&self) -> Balance {
        self.total_stake / 10
    }

    fn select_winner(&self) -> AccountId {
        let players: Vec<AccountId> = self.players.keys_as_vector().to_vec();
        assert!(!players.is_empty(), "No players to select a winner from");

        let random_seed = env::random_seed();
        let seed_array: [u8; 32] = {
            let mut seed = [0u8; 32];
            let bytes_to_copy = std::cmp::min(random_seed.len(), 32);
            seed[..bytes_to_copy].copy_from_slice(&random_seed[..bytes_to_copy]);
            seed
        };
        let mut rng = StdRng::from_seed(seed_array);
        let winner_index = rng.gen_range(0..players.len());
        players[winner_index].clone()
    }
}

#[ext_contract(ext_staking_contract)]
trait StakingContract {
    fn stake(&mut self, amount: U128);
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId};

    #[test]
    fn test_initialization() {
        let owner: AccountId = "owner.testnet".parse().unwrap();
        let context = VMContextBuilder::new()
            .signer_account_id(owner.clone())
            .build();
        testing_env!(context);

        let contract = Contract::new(
            owner,
            "staking.testnet".parse().unwrap(),
            U128(1_000_000),
        );
        assert_eq!(contract.min_deposit, 1_000_000);
    }
}