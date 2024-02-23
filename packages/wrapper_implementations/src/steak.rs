use basset::wrapper::ExecuteMsg;
use cavern_lsd_wrapper_token::msg::TokenInitMsg;
use cosmwasm_std::Binary;
use cosmwasm_std::DepsMut;
use cosmwasm_std::MessageInfo;
use cosmwasm_std::Response;
use cw20_base::msg::QueryMsg;
use cw20_base::ContractError;

use serde::Deserialize;
use std::marker::PhantomData;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::to_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::Deps;
use cosmwasm_std::Env;
use cosmwasm_std::QueryRequest;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use cosmwasm_std::WasmMsg;
use cosmwasm_std::WasmQuery;

use cw20::BalanceResponse;
use cw20::Cw20ExecuteMsg;
use cw20::Cw20QueryMsg;

use basset::external::LSDQueryMsg;
use basset::external::LSDStateResponseTrait;
use cavern_lsd_wrapper_token::trait_def::LSDHub;

use cosmwasm_std::Decimal;
#[cw_serde]
pub struct LsdContracts {
    pub hub: Addr,
    pub token: Addr,
}

#[cw_serde]
pub struct LsdContractsRaw {
    pub hub: String,
    pub token: String,
}

#[cw_serde]
pub struct SteakLSDHubMessage {
    pub lsd_contracts: LsdContracts,
}

#[cw_serde]
pub struct SteakLSDHub<T: for<'a> Deserialize<'a>> {
    pub types: Option<PhantomData<T>>,
    pub lsd_contracts: LsdContracts,
}

impl<T: LSDStateResponseTrait + for<'a> Deserialize<'a>> SteakLSDHub<T> {
    pub fn query_lsd_state(&self, deps: Deps, lsd_contracts: &LsdContracts) -> StdResult<T> {
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: lsd_contracts.hub.to_string(),
            msg: to_binary(&LSDQueryMsg::State {})?,
        }))
    }
}

impl<T: LSDStateResponseTrait + for<'a> Deserialize<'a>> LSDHub<LsdContractsRaw>
    for SteakLSDHub<T>
{
    fn instantiate_config(deps: Deps, config: LsdContractsRaw) -> StdResult<Self> {
        Ok(Self {
            types: None,
            lsd_contracts: LsdContracts {
                hub: deps.api.addr_validate(&config.hub)?,
                token: deps.api.addr_validate(&config.token)?,
            },
        })
    }

    fn query_exchange_rate(&self, deps: Deps, _env: Env) -> StdResult<Decimal> {
        // For steak based LSD, we query the corresponding luna value of that LSD
        // This is located in the state query of the LSD
        let lsd_state: T = self.query_lsd_state(deps, &self.lsd_contracts)?;

        Ok(lsd_state.exchange_rate())
    }

    fn get_balance(&self, deps: Deps, _env: Env, address: Addr) -> StdResult<Uint128> {
        let balance: BalanceResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: self.lsd_contracts.token.to_string(),
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
            contract_addr: self.lsd_contracts.token.to_string(),
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
            contract_addr: self.lsd_contracts.token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: to.to_string(),
                amount,
            })?,
            funds: vec![],
        })])
    }
}

pub type SteakInitMsg = TokenInitMsg<LsdContractsRaw>;

pub fn instantiate<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: SteakInitMsg,
) -> StdResult<Response> {
    cavern_lsd_wrapper_token::contract::instantiate::<LsdContractsRaw, SteakLSDHub<T>>(
        deps, env, info, msg,
    )
}

pub fn execute<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    cavern_lsd_wrapper_token::contract::execute::<LsdContractsRaw, SteakLSDHub<T>>(
        deps, env, info, msg,
    )
}

pub fn query<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    cavern_lsd_wrapper_token::contract::query::<LsdContractsRaw, SteakLSDHub<T>>(deps, env, msg)
}
