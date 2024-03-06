#[cfg(test)]
mod tests;

use basset::eris_lp::StateResponse;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};

use cw20::BalanceResponse;
use cw20::Cw20ExecuteMsg;
use cw20::Cw20QueryMsg;

use basset::wrapper::QueryMsg;
use cw20_base::ContractError;

use basset::wrapper::ExecuteMsg;

use cavern_lsd_wrapper_token_with_limit::msg::TokenInitMsg;
use cavern_lsd_wrapper_token_with_limit::trait_def::LSDHub;

#[cw_serde]
pub struct Contracts {
    pub token: Addr,
    pub hub: Addr,
}

#[cw_serde]
pub struct ContractsRaw {
    pub token: String,
    pub hub: String,
}

#[cw_serde]
pub struct AmphHub {
    pub contracts: Contracts,
}

impl LSDHub<ContractsRaw> for AmphHub {
    fn instantiate_config(deps: Deps, config: ContractsRaw) -> StdResult<Self> {
        Ok(Self {
            contracts: Contracts {
                hub: deps.api.addr_validate(&config.hub)?,
                token: deps.api.addr_validate(&config.token)?,
            },
        })
    }

    fn query_exchange_rate(&self, deps: Deps, _env: Env) -> StdResult<Decimal> {
        // For steak based LSD, we query the corresponding luna value of that LSD
        // This is located in the state query of the LSD
        let amp_token_state: StateResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: self.contracts.hub.to_string(),
                msg: to_binary(&basset::eris_lp::QueryMsg::State { addr: None })?,
            }))?;
        Ok(amp_token_state.exchange_rate)
    }

    fn get_balance(
        &self,
        deps: Deps,
        _env: Env,
        address: Addr,
        _funds: Vec<Coin>,
    ) -> StdResult<Uint128> {
        let balance: BalanceResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: self.contracts.token.to_string(),
                msg: to_binary(&Cw20QueryMsg::Balance {
                    address: address.to_string(),
                })?,
            }))?;
        Ok(balance.balance)
    }

    fn deposit_funds(
        &self,
        _deps: Deps,
        env: Env,
        _info: MessageInfo,
        amount: Uint128,
        from: Addr,
    ) -> StdResult<Vec<CosmosMsg>> {
        Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.contracts.token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: from.to_string(),
                recipient: env.contract.address.to_string(),
                amount,
            })?,
            funds: vec![],
        })])
    }

    fn send_funds(
        &self,
        _deps: Deps,
        _env: Env,
        amount: Uint128,
        to: Addr,
    ) -> StdResult<Vec<CosmosMsg>> {
        Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.contracts.token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: to.to_string(),
                amount,
            })?,
            funds: vec![],
        })])
    }
}

pub type SpectrumInitMsg = TokenInitMsg<ContractsRaw>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: SpectrumInitMsg,
) -> StdResult<Response> {
    cavern_lsd_wrapper_token_with_limit::contract::instantiate::<ContractsRaw, AmphHub>(
        deps, env, info, msg,
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    cavern_lsd_wrapper_token_with_limit::contract::execute::<ContractsRaw, AmphHub>(
        deps, env, info, msg,
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cavern_lsd_wrapper_token_with_limit::contract::query::<ContractsRaw, AmphHub>(deps, env, msg)
}
