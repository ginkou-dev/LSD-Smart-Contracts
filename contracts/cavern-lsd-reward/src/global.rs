use crate::querier::query_all_cw20_balances;
use crate::swap::Asset;
use cosmwasm_std::Coin;

use crate::state::read_config;

use basset::dex_router::AssetInfo;
use cosmwasm_std::{attr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

use crate::swap::create_swap_msgs;

/// Swap all native tokens to reward_denom
/// Only hub_contract is allowed to execute
#[allow(clippy::if_same_then_else)]
#[allow(clippy::needless_collect)]
pub fn execute_swap(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    if info.sender != config.hub_contract {
        return Err(StdError::generic_err("unauthorized"));
    }

    let contr_addr = env.contract.address.clone();

    // We start by swapping out all native coin balances
    let balances = deps.querier.query_all_balances(contr_addr)?;

    let reward_denom = config.clone().reward_denom;

    let native_swap_messages: Vec<CosmosMsg> = balances
        .iter()
        .filter(|x| reward_denom.clone() != x.denom)
        .map(|coin: &Coin| {
            create_swap_msgs(
                deps.as_ref(),
                env.clone(),
                Asset {
                    asset_info: AssetInfo::NativeToken {
                        denom: coin.denom.clone(),
                    },
                    amount: coin.amount,
                },
                config.reward_denom.clone(),
            )
        })
        .flat_map(|result| match result {
            Ok(vec) => vec.into_iter().map(Ok).collect(),
            Err(er) => vec![Err(er)],
        })
        .collect::<StdResult<Vec<CosmosMsg>>>()?;

    // Then we want to swap all cw20 balances we know into the stable denom
    let cw20_balances: Vec<Asset> = query_all_cw20_balances(
        deps.as_ref(),
        env.contract.address.clone(),
        &config.known_cw20_tokens,
    )?;
    let cw20_messages: Vec<CosmosMsg> = cw20_balances
        .iter()
        .filter(|asset| !asset.amount.is_zero())
        .map(|asset: &Asset| {
            create_swap_msgs(
                deps.as_ref(),
                env.clone(),
                Asset {
                    asset_info: asset.asset_info.clone(),
                    amount: asset.amount,
                },
                config.reward_denom.clone(),
            )
        })
        .flat_map(|result| match result {
            Ok(vec) => vec.into_iter().map(Ok).collect(),
            Err(er) => vec![Err(er)],
        })
        .collect::<StdResult<Vec<CosmosMsg>>>()?;

    let res = Response::new()
        .add_messages(native_swap_messages)
        .add_messages(cw20_messages)
        .add_attributes(vec![attr("action", "swap")]);

    Ok(res)
}
