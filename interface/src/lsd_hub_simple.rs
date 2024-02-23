use cw_orch::{
    interface,
    prelude::*,
};
use basset::hub::{
    ExecuteMsg, InstantiateMsg, QueryMsg,
};

use cavern_lsd_hub_simple::contract::{instantiate, execute, query, migrate};

use crate::WASM_SUFFIX;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct LsdHubSimple;

impl<Chain: CwEnv> Uploadable for LsdHubSimple<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path(&format!("cavern_lsd_hub_simple{}", WASM_SUFFIX))
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
