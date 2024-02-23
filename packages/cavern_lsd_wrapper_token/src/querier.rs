use serde::Deserialize;
use serde::Serialize;

use cosmwasm_std::Decimal;
use cosmwasm_std::Deps;
use cosmwasm_std::Env;
use cosmwasm_std::Uint128;

use cw20_base::contract::query_token_info;
use cw20_base::ContractError;

use crate::state::read_lsd_config;
use crate::state::WrapperState;
use crate::trait_def::LSDHub;

pub fn get_current_exchange_rate<
    I: for<'a> Deserialize<'a> + Serialize,
    T: LSDHub<I> + for<'b> Deserialize<'b> + Serialize,
>(
    deps: Deps,
    env: Env,
    state: &mut WrapperState,
) -> Result<Decimal, ContractError> {
    let lsd_config: T = read_lsd_config(deps.storage)?;
    let lsd_exchange_rate = lsd_config.query_exchange_rate(deps, env.clone())?;

    // We query how much lsd tokens the contract holds
    let balance: Uint128 = lsd_config.get_balance(deps, env.clone(), env.contract.address)?;

    // We now have the number of underlying lunas backing the token
    let luna_backing_token: Decimal = Decimal::from_ratio(balance, 1u128) * lsd_exchange_rate;

    // We can divide that by the number of issued tokens to get the exchange rate
    let total_wlsd_supply = query_token_info(deps)?.total_supply;

    state.lsd_exchange_rate = lsd_exchange_rate;
    state.wlsd_supply = total_wlsd_supply;
    state.backing_luna = luna_backing_token;
    state.lsd_balance = balance;

    // Luna / WLSD
    if total_wlsd_supply.is_zero() {
        Ok(Decimal::one())
    } else {
        Ok(luna_backing_token / total_wlsd_supply)
    }
}
