use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Coin, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use std::collections::HashMap;

pub static CONFIG_KEY: &[u8] = b"config";
pub static USCRT_DENOM: &str = "uscrt";


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ticket {
    pub id: u64,
    pub owner: CanonicalAddr
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub tickets: Vec<Ticket>,
    pub ticket_count: HashMap<String, u64>,
    pub contract_owner: CanonicalAddr,
    pub deposit: Uint128,
    pub start_time: u64,
    pub win_ticket: u64
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}
