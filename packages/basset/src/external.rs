use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum LSDQueryMsg {
    State {},
}

#[cw_serde]
pub struct LSDStateResponse {
    /// Total supply to the Steak token
    pub total_usteak: Uint128,
    /// Total amount of uluna staked
    pub total_uluna: Uint128,
    /// The exchange rate between usteak and uluna, in terms of uluna per usteak
    pub exchange_rate: Decimal,
    /// Staking rewards currently held by the contract that are ready to be reinvested
    pub unlocked_coins: Vec<Coin>,
}

pub trait LSDStateResponseTrait {
    fn exchange_rate(&self) -> Decimal;
}

impl LSDStateResponseTrait for LSDStateResponse {
    fn exchange_rate(&self) -> Decimal {
        self.exchange_rate
    }
}

// Spectrum token

#[cw_serde]
pub enum SpectrumExecuteMsg {
    Unbond { amount: Uint128 },
}

#[cw_serde]
pub enum SpectrumQueryMsg {
    State {},
    UserInfo { user: String, lp_token: String },
    RewardInfo { staker_addr: String },
}

#[cw_serde]
pub struct CTokenStateResponse {
    /// Total supply to the cToken
    pub total_bond_share: Uint128,
}

#[cw_serde]
pub struct UserInfoResponse {
    /// Total supply to the cToken
    pub bond_share: Uint128,
    pub bond_amount: Uint128,
    pub reward_indexes: Vec<(String, Decimal)>,
    pub pending_rewards: Vec<(String, Decimal)>,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct RewardInfoResponse {
    pub staker_addr: String,
    pub reward_info: RewardInfoResponseItem,
}

#[cw_serde]
pub struct RewardInfoResponseItem {
    /// The LP token contract address
    pub staking_token: String,
    /// The LP token amount bonded
    pub bond_amount: Uint128,
    /// The share of total LP token bonded
    pub bond_share: Uint128,
    /// The deposit amount
    pub deposit_amount: Uint128,
    /// The weighted average deposit time
    pub deposit_time: u64,
    /// The deposit cost
    pub deposit_costs: Vec<Uint128>,
}