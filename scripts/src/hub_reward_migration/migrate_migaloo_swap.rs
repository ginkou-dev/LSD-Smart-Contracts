

// We are migrating Whale related reward contracts so they don't include the max_spread field anymore

use cw_orch::prelude::{CwEnv, CwOrchUpload, ContractInstance, CwOrchMigrate, Addr, TxHandler, Empty};
use interface::LsdRewards;

pub const BONE_WHALE_REWARD: &str = "migaloo1m5yx7zdtdx6q8qd6njazu3uyk0drtcxhkh9cydym85ss5ktdppcswcztt9";
pub const AMP_WHALE_REWARD: &str = "migaloo1fjf4rnt9p2xa4uu73tsczve06stg8egvnhlg6nxxntfgc3pfv7jqd6hq4w";


pub fn upload_reward<Chain: CwEnv>(app: Chain) -> anyhow::Result<u64>{
    let reward_contract = LsdRewards::new("reward", app.clone());
    reward_contract.upload()?;
    Ok(reward_contract.code_id()?)
}

pub fn migrate_rewards<Chain: CwEnv>(mut app: Chain, sender: <Chain as TxHandler>::Sender) -> anyhow::Result<()>{
    // First we upload 
    let code_id = upload_reward(app.clone())?;

    app.set_sender(sender);
    let reward_contract = LsdRewards::new("reward", app.clone());
    
    // Then we migrate (this is permissioned)
    reward_contract.set_address(&Addr::unchecked(BONE_WHALE_REWARD));
    reward_contract.migrate(&Empty{}, code_id)?;

    reward_contract.set_address(&Addr::unchecked(AMP_WHALE_REWARD));
    reward_contract.migrate(&Empty{}, code_id)?;
    Ok(())
}
