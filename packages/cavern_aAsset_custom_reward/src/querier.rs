use crate::swap::Asset;
use basset::dex_router::AssetInfo;
use cosmwasm_std::to_binary;
use cosmwasm_std::Deps;
use cosmwasm_std::QueryRequest;
use cosmwasm_std::WasmQuery;
use cw20::BalanceResponse;
use cw20::Cw20QueryMsg;
use std::convert::TryInto;

use cosmwasm_std::Addr;

use cosmwasm_std::StdResult;

use cosmwasm_std::Uint256;

use cosmwasm_std::Uint128;

pub fn query_token_balance(
    deps: Deps,
    contract_addr: Addr,
    account_addr: Addr,
) -> StdResult<Uint256> {
    // load balance form the token contract
    let balance_response: BalanceResponse = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: account_addr.to_string(),
            })?,
        }))
        .unwrap_or_else(|_| BalanceResponse {
            balance: Uint128::zero(),
        });

    Ok(balance_response.balance.into())
}
pub fn query_all_cw20_balances(
    deps: Deps,
    contract_addr: Addr,
    tokens: &[Addr],
) -> StdResult<Vec<Asset>> {
    tokens
        .iter()
        .map(|token| {
            let result = query_token_balance(deps, token.clone(), contract_addr.clone());
            let asset_info = AssetInfo::Token {
                contract_addr: token.clone(),
            };
            result
                .map(|amount| Asset {
                    amount: amount.try_into().unwrap(),
                    asset_info: asset_info.clone(),
                })
                .or_else(|_| {
                    Ok(Asset {
                        amount: Uint128::zero(),
                        asset_info: asset_info.clone(),
                    })
                })
        })
        .collect()
}
