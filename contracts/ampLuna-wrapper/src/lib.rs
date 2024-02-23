use basset::external::LSDStateResponseTrait;
use basset::wrapper::ExecuteMsg;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use cosmwasm_std::Decimal;
use cosmwasm_std::Deps;
use cosmwasm_std::DepsMut;
use cosmwasm_std::Env;
use cosmwasm_std::MessageInfo;
use cosmwasm_std::Response;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use cosmwasm_std::{entry_point, Coin};

use cw20_base::msg::QueryMsg;
use cw20_base::ContractError;

use wrapper_implementations::steak;
#[cw_serde]
pub struct AmpLunaStateResponse {
    pub total_ustake: Uint128,
    pub total_uluna: Uint128,
    pub exchange_rate: Decimal,
    pub unlocked_coins: Vec<Coin>,
    pub unbonding: Uint128,
    pub available: Uint128,
    pub tvl_uluna: Uint128,
}

impl LSDStateResponseTrait for AmpLunaStateResponse {
    fn exchange_rate(&self) -> Decimal {
        self.exchange_rate
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: steak::SteakInitMsg,
) -> StdResult<Response> {
    steak::instantiate::<AmpLunaStateResponse>(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    steak::execute::<AmpLunaStateResponse>(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    steak::query::<AmpLunaStateResponse>(deps, env, msg)
}
