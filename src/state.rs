use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, HumanAddr, Coin, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

pub static CONFIG_KEY: &[u8] = b"config";
pub static USCRT_DENOM: &str = "uscrt";


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ticket {
    pub id: u64,
    pub owner: CanonicalAddr
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct History {
    pub end_time: u64,
    pub ticket: u64,
    pub address: HumanAddr,
    pub amount: Uint128
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub tickets: Vec<Ticket>,
    pub contract_owner: CanonicalAddr,
    pub deposit: Uint128,
    pub start_time: u64,
    pub win_ticket: u64,
    pub win_amount: Uint128,
    pub winner: CanonicalAddr,
    pub interval: u64,
    pub histories: Vec<History>
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}
