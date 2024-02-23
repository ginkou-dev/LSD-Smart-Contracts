use cosmwasm_schema::cw_serde;
use cw20::Cw20Coin;
use std::marker::PhantomData;

#[cw_serde]
pub struct TokenInitMsg<I> {
    pub types: Option<PhantomData<I>>,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,

    // only hub contract can call decompound
    pub hub_contract: String,

    pub lsd_config: I,
}
