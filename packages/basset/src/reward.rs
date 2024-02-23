use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal};

use cosmwasm_std::{Decimal256, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub hub_contract: String,
    pub reward_denom: String,
    pub astroport_addr: String,
    pub phoenix_addr: String,
    pub terraswap_addr: String,
    // Known tokens to swap from to the stable_token
    pub known_tokens: Vec<String>,
}

#[cw_serde]
#[cfg_attr(feature="interface", derive(cw_orch::ExecuteFns))]

pub enum ExecuteMsg {
    ////////////////////
    /// Owner's operations
    ///////////////////

    /// Swap all of the balances to uusd.
    SwapToRewardDenom {},

    /// Updates the contract config
    UpdateConfig {
        owner: Option<String>,
        custody_contract: Option<String>,
        known_tokens: Option<Vec<String>>,
        astroport_addr: Option<String>,
        phoenix_addr: Option<String>,
        terraswap_addr: Option<String>,
    },
    ////////////////////
    /// User's operations
    ///////////////////

    /// return the accrued reward in uusd to the user.
    ClaimRewards { recipient: Option<String> },
}

#[cw_serde]
#[cfg_attr(feature="interface", derive(cw_orch::QueryFns))]
#[derive(QueryResponses)]

pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AccruedRewardsResponse)]
    AccruedRewards { address: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub hub_contract: String,
    pub reward_denom: String,
}

#[cw_serde]
pub struct AccruedRewardsResponse {
    pub rewards: Uint128,
}

#[cw_serde]
pub struct HolderResponse {
    pub address: String,
    pub balance: Uint128,
    pub index: Decimal256,
    pub pending_rewards: Decimal256,
}

#[cw_serde]
pub struct HoldersResponse {
    pub holders: Vec<HolderResponse>,
}

#[cw_serde]
pub struct MigrateMsg {
    pub max_decompound_ratio: Option<Decimal>,
    pub hub_contract: Option<Addr>,
}
