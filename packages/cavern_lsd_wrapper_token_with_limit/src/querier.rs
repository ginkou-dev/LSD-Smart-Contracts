use cosmwasm_std::Coin;
use serde::Deserialize;
use serde::Serialize;

use cosmwasm_std::Decimal;
use cosmwasm_std::Deps;
use cosmwasm_std::Env;
use cosmwasm_std::Uint128;

use cw20_base::contract::query_token_info;
use cw20_base::ContractError;

use crate::contract::SECONDS_PER_YEAR;
use crate::state::{read_lsd_config, read_lsd_decompound_rate};
use crate::state::{DecompoundConfig, WrapperState};
use crate::state::{DECOMPOUND_CONFIG, DECOMPOUND_STATE};
use crate::trait_def::LSDHub;

/// Computes the exchange rate of the underlyingToken/Wrapped token
pub fn get_current_exchange_rate<
    I: for<'a> Deserialize<'a> + Serialize,
    T: LSDHub<I> + for<'b> Deserialize<'b> + Serialize,
>(
    deps: Deps,
    env: Env,
    state: &mut WrapperState,
) -> Result<Decimal, ContractError> {
    let lsd_config: T = read_lsd_config(deps.storage)?;
    let lsd_exchange_rate = lsd_config.query_exchange_rate(deps, env.clone())?; // This is the exchange rate underlyingToken/LSD

    // We query how much lsd tokens the contract holds
    let balance: Uint128 =
        lsd_config.get_balance(deps, env.clone(), env.contract.address, vec![])?;

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

/// Computes the exchange rate of the underlyingToken/Wrapped Token.
/// This function allows one to get the current value of the token with the current de-compound rate applied
pub fn get_expected_exchange_rate<
    I: for<'a> Deserialize<'a> + Serialize,
    T: LSDHub<I> + for<'b> Deserialize<'b> + Serialize,
>(
    deps: Deps,
    env: Env,
    state: &mut WrapperState,
) -> Result<Decimal, ContractError> {
    let exchange_rate = get_current_exchange_rate::<I, T>(deps, env.clone(), state)?;

    // If the exchange rate is lower than 1, we return it,
    // The token has had a slashing event
    if exchange_rate < Decimal::one() {
        return Ok(exchange_rate);
    }

    // Then if there is a maximum_decompound ratio, we try to get the expected exchange rate
    if let Some(max_decompound_ratio) = DECOMPOUND_CONFIG.load(deps.storage)?.max_decompound_ratio {
        let state = DECOMPOUND_STATE.load(deps.storage)?;

        let time_since_start =
            state.total_seconds + (env.block.time.seconds() - state.last_decompound.seconds());

        let mut expected_exchange_rate = (exchange_rate + state.ratio_sum)
            .checked_sub(
                max_decompound_ratio * Decimal::from_ratio(time_since_start, SECONDS_PER_YEAR),
            )
            .unwrap_or(Decimal::one());
        expected_exchange_rate = expected_exchange_rate.max(Decimal::one());

        return Ok(expected_exchange_rate);
    }

    Ok(exchange_rate)
}

/// Queries the exchange rate lsd <-> Wrapper token (how much wrapper token for 1 LSD amount)
/// This only requires querying the amount of LSD tokens locked in the contract
pub fn get_lsd_wrapper_exchange_rate<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: Deps,
    env: Env,
    funds: Vec<Coin>,
) -> Result<Decimal, ContractError> {
    let lsd_config: T = read_lsd_config(deps.storage)?;
    let total_lsd_balance =
        lsd_config.get_balance(deps, env.clone(), env.contract.address, funds)?;
    let total_supply = query_token_info(deps)?.total_supply;
    if total_lsd_balance.is_zero() || total_supply.is_zero() {
        return Ok(Decimal::one());
    }
    Ok(Decimal::from_ratio(total_supply, total_lsd_balance))
}

pub fn get_lsd_wrapper_decompound_rate(
    deps: Deps,
    env: Env,
) -> Result<DecompoundConfig, ContractError> {
    let decompound_rate: DecompoundConfig = read_lsd_decompound_rate(deps.storage)?;
    Ok(decompound_rate)
}

pub fn query_mint_amount<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: Deps,
    env: Env,
    amount: Uint128,
) -> Result<Uint128, ContractError> {
    // In order to mint, we need to transfer the underlying lsd asset to the contract
    // Any sender can call this function as long as they have the sufficient lsd balance
    let lsd_config: T = read_lsd_config(deps.storage)?;
    let exchange_rate = get_lsd_wrapper_exchange_rate::<I, T>(deps, env.clone(), vec![])?;
    let mint_amount = amount * exchange_rate;
    Ok(mint_amount * Uint128::one())
}
