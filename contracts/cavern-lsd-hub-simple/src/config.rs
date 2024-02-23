use crate::state::CONFIG;
use basset::hub::Config;
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

/// Update the config. Update the owner, reward and token contracts.
/// Only creator/owner is allowed to execute
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
    reward_contract: Option<String>,
    token_contract: Option<String>,
) -> StdResult<Response> {
    // only owner must be able to send this message.
    let conf = CONFIG.load(deps.storage)?;
    if info.sender != conf.creator {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(o) = owner {
        let owner_raw = deps.api.addr_validate(o.as_str())?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<Config> {
            last_config.creator = owner_raw;
            Ok(last_config)
        })?;
    }
    if let Some(reward) = reward_contract {
        let reward_raw = deps.api.addr_validate(reward.as_str())?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<Config> {
            last_config.reward_contract = Some(reward_raw);
            Ok(last_config)
        })?;
    }

    if let Some(token) = token_contract {
        let token_raw = deps.api.addr_validate(token.as_str())?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<Config> {
            last_config.token_contract = Some(token_raw);
            Ok(last_config)
        })?;
    }

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}
