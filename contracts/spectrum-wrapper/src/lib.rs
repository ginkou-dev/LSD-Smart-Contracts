#[cfg(test)]
mod tests;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};

use cw20::BalanceResponse;
use cw20::Cw20ExecuteMsg;
use cw20::Cw20QueryMsg;
use cw20_base::ContractError;

use basset::external::CTokenStateResponse;
use basset::external::SpectrumQueryMsg;
use basset::external::UserInfoResponse;
use basset::wrapper::{ExecuteMsg, QueryMsg};

use cavern_lsd_wrapper_token_with_limit::msg::TokenInitMsg;
use cavern_lsd_wrapper_token_with_limit::trait_def::LSDHub;

#[cw_serde]
pub struct Contracts {
    pub token: Addr,
    pub underlying_token: Addr,
    pub generator: Addr,
}

#[cw_serde]
pub struct ContractsRaw {
    pub token: String,
    pub generator: String,
    pub underlying_token: String,
}

#[cw_serde]
pub struct SpectrumHub {
    pub contracts: Contracts,
}

impl SpectrumHub {
    pub fn query_total_bond_share(&self, deps: Deps, contracts: &Contracts) -> StdResult<Uint128> {
        let c_token_state: CTokenStateResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contracts.token.to_string(),
                msg: to_binary(&SpectrumQueryMsg::State {})?,
            }))?;
        Ok(c_token_state.total_bond_share)
    }
    pub fn query_total_bond_amount(&self, deps: Deps, contracts: &Contracts) -> StdResult<Uint128> {
        let user_info: UserInfoResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contracts.generator.to_string(),
                msg: to_binary(&SpectrumQueryMsg::UserInfo {
                    user: contracts.token.to_string(),
                    lp_token: contracts.underlying_token.to_string(),
                })?,
            }))?;
        Ok(user_info.bond_amount)
    }
}

impl LSDHub<ContractsRaw> for SpectrumHub {
    fn instantiate_config(deps: Deps, config: ContractsRaw) -> StdResult<Self> {
        Ok(Self {
            contracts: Contracts {
                generator: deps.api.addr_validate(&config.generator)?,
                token: deps.api.addr_validate(&config.token)?,
                underlying_token: deps.api.addr_validate(&config.underlying_token)?,
            },
        })
    }

    fn query_exchange_rate(&self, deps: Deps, _env: Env) -> StdResult<Decimal> {
        // For steak based LSD, we query the corresponding underlying token value of that LSD
        // This is located in the state query of the LSD
        let total_bond_share = self.query_total_bond_share(deps, &self.contracts)?;
        let total_bond_amount = self.query_total_bond_amount(deps, &self.contracts)?;

        Ok(Decimal::from_ratio(total_bond_amount, total_bond_share))
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
    cavern_lsd_wrapper_token_with_limit::contract::instantiate::<ContractsRaw, SpectrumHub>(
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
    cavern_lsd_wrapper_token_with_limit::contract::execute::<ContractsRaw, SpectrumHub>(
        deps, env, info, msg,
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cavern_lsd_wrapper_token_with_limit::contract::query::<ContractsRaw, SpectrumHub>(
        deps, env, msg,
    )
}
