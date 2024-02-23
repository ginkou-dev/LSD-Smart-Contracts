use cw_orch::{
    interface,
    prelude::*,
};
pub use basset::reward::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};


use cavern_lsd_reward::contract::{instantiate, execute, query, migrate};

use crate::WASM_SUFFIX;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct LsdRewards;

impl<Chain: CwEnv> Uploadable for LsdRewards<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path(&format!("cavern_lsd_reward{}", WASM_SUFFIX))
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                execute,
                instantiate,
                query,
            )
            .with_migrate(migrate)
        )
    }
}
