use cosmwasm_std::{
    coin, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HumanAddr, InitResponse, Querier, StdError, StdResult, Storage, Uint128
};

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use lazy_static::lazy_static;

use crate::msg::{HandleMsg, InitMsg, QueryMsg, CountResponse, StateResponse, HashObj};
use crate::state::{config, config_read, State, Ticket, USCRT_DENOM};
// use fastrand;

const INTERVAL:u64 = 604800;
const MAXTICKET:u64 = 99;
const FIRSTSUNDAY:u64 = 316800;

lazy_static! {
    static ref ZERO_ADDRESS: CanonicalAddr = CanonicalAddr(Binary(vec![0; 8]));
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    
    let tickets = Vec::<Ticket>::new();

    // 0 : 1970.1.1 00:00:00 Thu
    // must add 3 days and 16 hours then get first sunday 16:00 => 316800
    // we must set the start_time to the nearest sunday 1600
    // (env.block.time - 316800) % INTERVAL * INTERVAL + 316800

    let start_time = (env.block.time - FIRSTSUNDAY) / INTERVAL * INTERVAL + FIRSTSUNDAY;

    // Create state
    let state = State {
        tickets,
        contract_owner: deps.api.canonical_address(&env.message.sender)?,
        deposit: Uint128::zero(),
        start_time,
        win_ticket:0u64
    };
    
    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        
        HandleMsg::BuyTicket { ticket_amount } => buy_ticket(deps, env, ticket_amount),
        HandleMsg::NewRound {} => new_round(deps, env),
        HandleMsg::SetConstant {house_addr} => set_constant(deps, env, &house_addr)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::TicketsOf { owner } => to_binary(&tickets_of(deps, &owner)),
        QueryMsg::TotalBalance { } => to_binary(&total_balance(deps)),
        QueryMsg::IsFinished {} => to_binary(&is_finished(deps)),
        QueryMsg::Winner{} => to_binary(&get_winner(deps)),
        QueryMsg::TotalState{} => to_binary(&total_state(deps))
                
    }
}

fn throw_gen_err(msg: String) -> StdError {
    StdError::GenericErr {
        msg,
        backtrace: None,
    }
}

// fn is_owner_or_approved(item: &Ticket, addr: &CanonicalAddr) -> bool {
//     addr == &item.owner || item.approved.clone().iter().any(|i| i == addr)
// }

fn is_token_id_valid(token_id: u64, state: &State) -> bool {
    (token_id as usize) < state.tickets.len()
}

// fn perform_transfer<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     to: &CanonicalAddr,
//     token_id: u32,
// ) -> StdResult<State> {
//     config(&mut deps.storage).update(|mut state| {
//         state.items[token_id as usize].owner = to.clone();
//         Ok(state)
//     })
// }

fn buy_ticket<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    ticket_amount: u64,
) -> StdResult<HandleResponse> {
    
    let mut state = config(&mut deps.storage).load()?;
    if env.message.sent_funds.is_empty() {
        return Err(throw_gen_err(format!("You can't get tickets for free!")));
    }

    let sent_funds: Coin = env.message.sent_funds[0].clone();

    let tamount:u128 = Uint128::from(ticket_amount).u128();
    if sent_funds.amount.u128() < tamount * 1000000 {
        return Err(throw_gen_err(format!(
            "You sent {:?} funds, it is not enough!",
            sent_funds.amount
        )));
    }
    
    //check deadline is passed and then call new_round
    if (env.block.time - state.start_time) > INTERVAL {
        new_round(deps, env.clone());
    }
    //End check

    let key:String = String::from(env.message.sender.as_str());
    let mut curamount:u64 = 0;
    
    for ticket in state.tickets.clone() {
        if env.message.sender == deps.api.human_address(&ticket.owner)? {
            curamount = curamount + 1;
        }
    }


    if curamount + ticket_amount > MAXTICKET {
        return Err(throw_gen_err(format!(
            "You can buy {:?} tickets at max!",
            MAXTICKET - curamount
        )));
    }
    
    // config(&mut deps.storage).update(|mut state| {
    //     state.deposit.0 += tamount;
    //     Ok(state)
    // })?;
    state.deposit.0 += sent_funds.amount.u128();

    let cnt:u64 = state.tickets.len() as u64;
    for i in 0..ticket_amount {
        state.tickets.push(Ticket {
            id: cnt + i,
            owner: deps.api.canonical_address(&env.message.sender)?,
        });
    }
    config(&mut deps.storage).save(&state)?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

fn set_constant<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    house_addr:&HumanAddr
) -> StdResult<HandleResponse> {
    let mut state = config(&mut deps.storage).load()?;
    state.contract_owner = deps.api.canonical_address(house_addr)?;
    config(&mut deps.storage).save(&state)?;

    let messages: Vec<CosmosMsg> = vec![];
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None,
    })
}
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn new_round<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env
) -> StdResult<HandleResponse> {
    let mut state = config(&mut deps.storage).load()?;
    let contract_addr: HumanAddr = deps.api.human_address(&deps.api.canonical_address(&env.contract.address)?)?;

    let ticket_count:u64 = state.tickets.len() as u64;
    if ticket_count == 0 {
        return Err(throw_gen_err(format!(
            "No tickets are sold!"
        )));
    }

    // fastrand::seed(env.block.time);
    // let mut rng = fastrand::Rng::new();
    // let rnd_ticket = rng.u64((0..ticketcount - 1));

    let mut str = String::from("");
    
    for ticket in state.tickets.clone() {
        str.push_str(deps.api.human_address(&ticket.owner)?.as_str());
    }
    let obj = HashObj {
        time: env.block.time,
        ticket_count,
        tickets: str
    };
    
    let rnd_ticket = calculate_hash(&obj) % ticket_count ;
    // let rnd_ticket = ((env.block.time % 100) * (ticketcount + env.block.time % 53) * (ticketcount + env.block.time % 37)) % ticketcount;
    let win_addr = state.tickets[rnd_ticket as usize].owner.clone();
    
    let winamount = state.deposit.u128().checked_mul(8).unwrap().checked_div(10).unwrap();
    let houseamount = state.deposit.u128() - winamount;

    let mut messages: Vec<CosmosMsg> = vec![];
    
    if state.deposit.u128() > 0 {
        //Send to winner
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: contract_addr.clone(),
            to_address: deps.api.human_address(&win_addr)?,
            amount: vec![Coin {
                denom: USCRT_DENOM.to_string(),
                amount: Uint128::from(winamount),
            }],
        }));
    
        //Send to house
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: contract_addr.clone(),
            to_address: deps.api.human_address(&state.contract_owner)?,
            amount: vec![Coin {
                denom: USCRT_DENOM.to_string(),
                amount: Uint128::from(houseamount),
            }],
        }));
    }
    state.tickets = Vec::<Ticket>::new();
    state.deposit = Uint128::zero();
    state.start_time = (env.block.time - FIRSTSUNDAY) / INTERVAL * INTERVAL + FIRSTSUNDAY;
    state.win_ticket = rnd_ticket;

    config(&mut deps.storage).save(&state)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None,
    })
}

fn tickets_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: &HumanAddr,
) -> StdResult<u64> {
    let owner_addr_raw = deps.api.canonical_address(&owner)?;

    if owner_addr_raw == *ZERO_ADDRESS {
        return Err(throw_gen_err("Can't query the zero address!".to_string()));
    }

    let state = config_read(&deps.storage).load()?;

    let mut curamount:u64 = 0u64;
    
    for ticket in state.tickets.clone() {
        if owner == &deps.api.human_address(&ticket.owner)? {
            curamount = curamount + 1;
        }
    }
    
    Ok(curamount)
}

fn is_finished<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<bool> {
    // let start_time = (env.block.time - FIRSTSUNDAY) / INTERVAL * INTERVAL + FIRSTSUNDAY;
    // Ok(start_time + INTERVAL > env.block.time)
    Ok(true)
}

fn get_winner<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<u64> {
    let state = config_read(&deps.storage).load()?;
    
    Ok(state.win_ticket)
}

fn total_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<u128> {
    let state = config_read(&deps.storage).load()?;
    
    Ok(state.deposit.u128())
}

fn total_state<S: Storage, A: Api, Q: Querier>
    (deps: &Extern<S, A, Q>
) -> StdResult<StateResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(StateResponse {
        tickets: state.tickets,
        contract_owner: deps.api.human_address(&state.contract_owner)?,
        deposit: state.deposit,
        start_time: state.start_time,
        win_ticket: state.win_ticket
    })
}


