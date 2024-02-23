use crate::trait_def::LSDHub;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cosmwasm_std::{Addr, StdResult, Storage};
use cosmwasm_std::{Decimal, Timestamp};
use serde::Deserialize;
use serde::Serialize;
//use cosmwasm_storage::{singleton, singleton_read};
use cw_storage_plus::Item;

pub const LSD_CONFIG_KEY: &str = "lcd_config";
pub const DECOMPOUND_CONFIG_KEY: &str = "decompound_config";
pub const HUB_CONTRACT_KEY: Item<Addr> = Item::new("\u{0}\u{c}hub_contract");

// We need to save the last rates that were decompounded in the past
pub const DECOMPOUND_STATE: Item<DecompoundState> = Item::new("decompound_state");
pub const DECOMPOUND_CONFIG: Item<DecompoundConfig> = Item::new("decompound_config");

#[cw_serde]
pub struct LsdContracts {
    pub hub: Addr,
    pub token: Addr,
}

#[cw_serde]
pub struct DecompoundState {
    pub ratio_sum: Decimal,
    pub total_seconds: u64,
    pub last_decompound: Timestamp,
}

#[cw_serde]
pub struct DecompoundConfig {
    pub max_decompound_ratio: Option<Decimal>,
}


// meta is the token definition as well as the total_supply
pub fn read_hub_contract(storage: &dyn Storage) -> StdResult<Addr> {
    HUB_CONTRACT_KEY.load(storage)
}

pub fn store_hub_contract(storage: &mut dyn Storage, hub_contract: &Addr) -> StdResult<()> {
    HUB_CONTRACT_KEY.save(storage, hub_contract)
}

// meta is the token definition as well as the total_supply
pub fn read_lsd_config<T: for<'a> Deserialize<'a> + Serialize>(
    storage: &dyn Storage,
) -> StdResult<T> {
    Item::new(LSD_CONFIG_KEY).load(storage)
}

pub fn read_lsd_decompound_rate(
    storage: &dyn Storage,
) -> StdResult<DecompoundConfig> {
    Item::new(DECOMPOUND_CONFIG_KEY).load(storage)
}

pub fn store_lsd_config<
    I: Serialize + for<'a> Deserialize<'a>,
    T: LSDHub<I> + for<'b> Deserialize<'b> + Serialize,
>(
    storage: &mut dyn Storage,
    lsd_config: &T,
) -> StdResult<()> {
    Item::new(LSD_CONFIG_KEY).save(storage, lsd_config)
}

#[cw_serde]
#[derive(Default)]
pub struct WrapperState {
    pub lsd_exchange_rate: Decimal,
    pub wlsd_supply: Uint128,
    pub backing_luna: Decimal,
    pub lsd_balance: Uint128,
}

#[cfg(test)]
mod test {
    use super::*;

    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Api, StdResult, Storage};
    use cosmwasm_storage::{singleton, singleton_read};

    pub static HUB_CONTRACT: &[u8] = b"hub_contract";

    pub fn store_hub(storage: &mut dyn Storage, params: &Addr) -> StdResult<()> {
        singleton(storage, HUB_CONTRACT).save(params)
    }
    pub fn read_hub(storage: &dyn Storage) -> StdResult<Addr> {
        singleton_read(storage, HUB_CONTRACT).load()
    }

    #[test]
    fn hub_legacy_compatibility() {
        let mut deps = mock_dependencies();
        store_hub(&mut deps.storage, &deps.api.addr_validate("hub").unwrap()).unwrap();

        assert_eq!(
            HUB_CONTRACT_KEY.load(&deps.storage).unwrap(),
            read_hub(&deps.storage).unwrap()
        );
    }
}
