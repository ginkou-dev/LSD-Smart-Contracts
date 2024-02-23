use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use cw20::Cw20Coin;
use std::marker::PhantomData;

#[cw_serde]
pub struct TokenInitMsg<I> {
    pub types: Option<PhantomData<I>>,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,

    // Maximum decompound ratio per year
    pub max_decompound_ratio: Option<Decimal>,

    // only hub contract can call decompound
    pub hub_contract: String,

    pub lsd_config: I,
}
