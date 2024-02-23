use crate::oracle::PriceResponse;
use crate::oracle::QueryMsg as OracleQueryMsg;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::to_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::Deps;
use cosmwasm_std::QueryRequest;
use cosmwasm_std::StdError;
use cosmwasm_std::StdResult;
use cosmwasm_std::WasmQuery;

#[cw_serde]
pub struct TimeConstraints {
    pub block_time: u64,
    pub valid_timeframe: u64,
}

pub fn query_price(
    deps: Deps,
    oracle_addr: Addr,
    base: String,
    quote: String,
    time_contraints: Option<TimeConstraints>,
) -> StdResult<PriceResponse> {
    let oracle_price: PriceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: oracle_addr.to_string(),
            msg: to_binary(&OracleQueryMsg::Price { base, quote })?,
        }))?;

    if let Some(time_contraints) = time_contraints {
        let valid_update_time = time_contraints.block_time - time_contraints.valid_timeframe;
        if oracle_price.last_updated_base < valid_update_time
            || oracle_price.last_updated_quote < valid_update_time
        {
            return Err(StdError::generic_err("Price is too old"));
        }
    }

    Ok(oracle_price)
}
