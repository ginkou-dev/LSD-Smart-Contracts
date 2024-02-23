use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal256, Deps, Order, StdResult, Storage, Uint128};
use serde::{de::DeserializeOwned, Serialize};

use basset::reward::HolderResponse;
use cw_storage_plus::{Bound, Item, Map};

pub const STATE: Item<State> = Item::new("\u{0}\u{5}state");
pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const HOLDERS: Map<&Addr, Holder> = Map::new("holders");

// New mecanism specific variables
pub const SWAP_CONFIG: Item<SwapConfig> = Item::new("swap_config");

#[cw_serde]
pub struct SwapConfig {
    pub astroport_addr: Addr,
    pub phoenix_addr: Addr,
    pub terraswap_addr: Addr,
}
// End

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub hub_contract: Addr,
    pub custody_contract: Option<Addr>,
    pub reward_denom: String,

    pub known_cw20_tokens: Vec<Addr>,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_retrieve_config<I: Serialize + DeserializeOwned>(
    storage: &mut dyn Storage,
    config: &I,
) -> StdResult<()> {
    let retrieve_config: Item<I> = Item::new("\u{0}\u{6}retrieve_config");
    retrieve_config.save(storage, config)
}

pub fn read_retrieve_config<I: Serialize + DeserializeOwned>(
    storage: &dyn Storage,
) -> StdResult<I> {
    let retrieve_config: Item<I> = Item::new("\u{0}\u{6}retrieve_config");
    retrieve_config.load(storage)
}

#[cw_serde]
pub struct State {
    pub global_index: Decimal256,
    pub total_balance: Uint128,
    pub prev_reward_balance: Uint128,
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

#[cw_serde]
pub struct Holder {
    pub balance: Uint128,
    pub index: Decimal256,
    pub pending_rewards: Decimal256,
}

// This is similar to HashMap<holder's address, Hodler>
pub fn store_holder(
    storage: &mut dyn Storage,
    holder_address: &Addr,
    holder: &Holder,
) -> StdResult<()> {
    HOLDERS.save(storage, holder_address, holder)
}

pub fn read_holder(storage: &dyn Storage, holder_address: &Addr) -> StdResult<Holder> {
    let res = HOLDERS.may_load(storage, holder_address)?;
    match res {
        Some(holder) => Ok(holder),
        None => Ok(Holder {
            balance: Uint128::zero(),
            index: Decimal256::zero(),
            pending_rewards: Decimal256::zero(),
        }),
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_holders(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<HolderResponse>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));

    HOLDERS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|elem| {
            let (k, v) = elem?;
            let address: String = k.to_string();
            Ok(HolderResponse {
                address,
                balance: v.balance,
                index: v.index,
                pending_rewards: v.pending_rewards,
            })
        })
        .collect()
}
