use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};

pub type UnbondRequest = Vec<(u64, Uint128)>;

#[cw_serde]
pub struct OldInstantiateMsg {
    pub reward_denom: String
}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(Default)]
pub struct State {
    pub exchange_rate: Decimal,
    pub total_bond_amount: Uint128,
    pub last_index_modification: u64,
    pub prev_hub_balance: Uint128,
}

#[cw_serde]
pub struct Config {
    pub creator: Addr,
    pub reward_contract: Option<Addr>,
    pub token_contract: Option<Addr>, // This is the address of the LSD Wrapper
}

impl State {
    pub fn update_exchange_rate(&mut self, total_issued: Uint128, requested_with_fee: Uint128) {
        let actual_supply = total_issued + requested_with_fee;
        if self.total_bond_amount.is_zero() || actual_supply.is_zero() {
            self.exchange_rate = Decimal::one()
        } else {
            self.exchange_rate = Decimal::from_ratio(self.total_bond_amount, actual_supply);
        }
    }
}

#[cw_serde]
#[cfg_attr(feature="interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    ////////////////////
    /// Owner's operations
    ////////////////////

    /// Set the owner
    UpdateConfig {
        owner: Option<String>,
        reward_contract: Option<String>,
        token_contract: Option<String>,
    },

    ////////////////////
    /// User's operations
    ////////////////////

    /// Update global index
    UpdateGlobalIndex {},
    // Check whether the slashing has happened or not
    //CheckSlashing {},
}

#[cw_serde]
#[cfg_attr(feature="interface", derive(cw_orch::QueryFns))]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    State {},
    #[returns(Parameters)]
    Parameters {},
}

#[cw_serde]
pub enum Cw20HookMsg {
    Unbond {},
}

#[cw_serde]
pub struct UnbondHistory {
    pub batch_id: u64,
    pub time: u64,
    pub amount: Uint128,
    pub applied_exchange_rate: Decimal,
    pub withdraw_rate: Decimal,
    pub released: bool,
}

#[cw_serde]
pub struct StateResponse {
    pub exchange_rate: Decimal,
    pub total_bond_amount: Uint128,
    pub last_index_modification: u64,
    pub prev_hub_balance: Uint128,
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub reward_contract: Option<String>,
    pub token_contract: Option<String>,
}

#[cw_serde]
pub struct WhitelistedValidatorsResponse {
    pub validators: Vec<String>,
}

#[cw_serde]
pub struct CurrentBatchResponse {
    pub id: u64,
    pub requested_with_fee: Uint128,
}

#[cw_serde]
pub struct WithdrawableUnbondedResponse {
    pub withdrawable: Uint128,
}
#[cw_serde]
pub struct UnbondRequestsResponse {
    pub address: String,
    pub requests: UnbondRequest,
}

#[cw_serde]
pub struct AllHistoryResponse {
    pub history: Vec<UnbondHistory>,
}


#[cw_serde]
pub struct Parameters {
    pub reward_denom: String,
}