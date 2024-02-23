pub mod distant_lsd_wrapper;
pub use distant_lsd_wrapper::LsdWrapper;
pub use basset::wrapper::{
    ExecuteMsgFns as WrapperExecuteMsgFns, QueryMsgFns as WrapperQueryMsgFns
};

pub mod lsd_hub;
pub use lsd_hub::LsdHub;
pub use basset::hub::{
    ExecuteMsgFns as HubExecuteMsgFns, QueryMsgFns as HubQueryMsgFns
};

pub mod lsd_hub_simple;
pub use lsd_hub_simple::LsdHubSimple;


pub mod lsd_reward;
pub use lsd_reward::LsdRewards;
pub use basset::reward::{
    ExecuteMsgFns as RewardExecuteMsgFns, QueryMsgFns as RewardQueryMsgFns,
    ExecuteMsg as RewardExecuteMsg, QueryMsg as RewardQueryMsg
};


pub const WASM_SUFFIX: &str = "";
// pub const WASM_SUFFIX: &str = "-x86_64";