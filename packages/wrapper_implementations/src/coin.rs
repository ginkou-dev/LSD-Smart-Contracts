use basset::price_querier::query_price;
use cosmwasm_std::BalanceResponse;
use cosmwasm_std::BankMsg;
use cosmwasm_std::BankQuery;
use cosmwasm_std::Binary;
use cosmwasm_std::Coin;
use std::convert::TryInto;

use basset::wrapper::ExecuteMsg;
use cavern_lsd_wrapper_token::msg::TokenInitMsg;
use cosmwasm_std::DepsMut;
use cosmwasm_std::MessageInfo;
use cosmwasm_std::Response;
use cosmwasm_std::StdError;
use cw20_base::msg::QueryMsg;
use cw20_base::ContractError;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::CosmosMsg;

use cosmwasm_std::Env;
use cosmwasm_std::QueryRequest;
use cosmwasm_std::Uint128;

use cosmwasm_std::Deps;
use cosmwasm_std::StdResult;

use cavern_lsd_wrapper_token::trait_def::LSDHub;

use cosmwasm_std::Decimal;

#[cw_serde]
pub struct StrideLSDConfigRaw {
    pub denom: String,
    pub underlying_token_denom: String,
    pub oracle_contract: String,
}

#[cw_serde]
pub struct StrideLSDConfig {
    pub denom: String,
    pub underlying_token_denom: String,
    pub oracle_contract: Addr,
}

impl LSDHub<StrideLSDConfigRaw> for StrideLSDConfig {
    fn instantiate_config(deps: Deps, config: StrideLSDConfigRaw) -> StdResult<Self> {
        Ok(Self {
            denom: config.denom,
            oracle_contract: deps.api.addr_validate(&config.oracle_contract)?,
            underlying_token_denom: config.underlying_token_denom,
        })
    }

    fn query_exchange_rate(&self, deps: Deps, _env: Env) -> StdResult<Decimal> {
        // For stride based tokens, the token is a native token and the exchange rate is not queryable on Terra
        // Therefore, we need to have an external oracle feed us the token prices
        let exchange_rate = query_price(
            deps,
            self.oracle_contract.clone(),
            self.denom.clone(),
            self.underlying_token_denom.clone(),
            None,
        )?
        .rate;
        let uint128_atomics: Uint128 = exchange_rate.atomics().try_into()?;
        // Decimal256 and Decimal have the same decimal places.
        Decimal::from_atomics(uint128_atomics, exchange_rate.decimal_places())
            .map_err(|e| StdError::generic_err(e.to_string()))
    }

    fn get_balance(&self, deps: Deps, _env: Env, address: Addr) -> StdResult<Uint128> {
        let balance: BalanceResponse =
            deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
                address: address.to_string(),
                denom: self.denom.to_string(),
            }))?;

        Ok(balance.amount.amount)
    }

    fn deposit_funds(
        &self,
        _deps: Deps,
        _env: Env,
        info: MessageInfo,
        amount: Uint128,
        _from: Addr,
    ) -> StdResult<Vec<CosmosMsg>> {
        if info.funds.len() != 1
            || info.funds[0].denom != self.denom
            || info.funds[0].amount < amount
        {
            return Err(StdError::generic_err(format!(
                "You need to deposit the right funds, deposited {:?}, needed {}{}",
                info.funds, amount, self.denom
            )));
        }

        Ok(vec![])
    }

    fn send_funds(
        &self,
        _deps: Deps,
        _env: Env,
        amount: Uint128,
        to: Addr,
    ) -> StdResult<Vec<CosmosMsg>> {
        Ok(vec![CosmosMsg::Bank(BankMsg::Send {
            to_address: to.to_string(),
            amount: vec![Coin {
                denom: self.denom.clone(),
                amount,
            }],
        })])
    }
}

pub type StrideInitMsg = TokenInitMsg<StrideLSDConfigRaw>;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: StrideInitMsg,
) -> StdResult<Response> {
    cavern_lsd_wrapper_token::contract::instantiate::<StrideLSDConfigRaw, StrideLSDConfig>(
        deps, env, info, msg,
    )
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    cavern_lsd_wrapper_token::contract::execute::<StrideLSDConfigRaw, StrideLSDConfig>(
        deps, env, info, msg,
    )
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cavern_lsd_wrapper_token::contract::query::<StrideLSDConfigRaw, StrideLSDConfig>(deps, env, msg)
}
