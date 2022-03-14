use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{CanonicalAddr, Coin, Storage, Uint128};

use crate::state::{config, config_read, State, Ticket, History, USCRT_DENOM};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub interval:u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    
    BuyTicket {
        ticket_amount: u64,
    },
    NewRound {},
    SetConstant {
        house_addr: HumanAddr
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    TicketsOf { owner: HumanAddr },
    TotalBalance { },
    IsFinished { },
    Winner { },
    TotalState { },
    Histories {}
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Hash)]
pub struct HashObj {
    pub time: u64,
    pub ticket_count: u64,
    pub tickets: String
}


// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub tickets: Vec<Ticket>,
    pub contract_owner: HumanAddr,
    pub deposit: Uint128,
    pub start_time: u64,
    pub win_ticket: u64,
    pub win_amount: Uint128,
    pub winner: HumanAddr
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HistoryResponse {
    pub histories: Vec<History>
}
