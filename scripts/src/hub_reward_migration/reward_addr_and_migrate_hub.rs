

// We are migrating Whale related reward contracts so they don't include the max_spread field anymore

use cw_orch::prelude::{CwEnv, CwOrchUpload, ContractInstance, CwOrchMigrate, Addr, Empty};
use interface::{LsdHubSimple, HubExecuteMsgFns};

pub const AMP_LUNA_HUB: &str = "migaloo1sv2zgqr5u4lwns80k07a8m9vfyf65hq64fklhxqwg8dslv2nqpwsddjkel";
pub const B_LUNA_HUB  : &str ="migaloo1kw7ga0w4c3hfskc6zc2pdc0x6qwvznrstvewzn6w5chsc249sl6sgqzwsw";
pub const AMP_ROAR_HUB: &str = "migaloo1exx0zzl003kgva0f5qmwdelnse98am5mrshp52w9vy8zqdpk8d3q8xq9py";
pub const FEE_COLLECTOR: &str = "migaloo13uf6cv8htse7dkcuykajr6e25czxcxct8pu2mnhq8zyr2hr0vxkqjwgvhm";

pub fn upload_hub_simple<Chain: CwEnv>(app: Chain) -> anyhow::Result<u64>{
    let hub_contract: LsdHubSimple<Chain> = LsdHubSimple::new("hub", app.clone());
    hub_contract.upload()?;
    Ok(hub_contract.code_id()?)
}

pub fn migrate_hub_and_change_reward<Chain: CwEnv>(mut app: Chain, sender: Chain::Sender) -> anyhow::Result<()>{
    // First we upload 
    let code_id = upload_hub_simple(app.clone())?;

    app.set_sender(sender);
    // Then we migrate (this is permissioned)
    migrate_and_reward(app.clone(), code_id, AMP_LUNA_HUB)?;
    migrate_and_reward(app.clone(), code_id, B_LUNA_HUB)?;
    migrate_and_reward(app.clone(), code_id, AMP_ROAR_HUB)?;

    Ok(())
}

fn migrate_and_reward<Chain: CwEnv>(app: Chain, code_id: u64, contract: &str)  -> anyhow::Result<()>{
    let hub_contract = LsdHubSimple::new("hub", app.clone());
    hub_contract.set_address(&Addr::unchecked(contract));
    hub_contract.migrate(&Empty{}, code_id)?;

    // Then we chage the reward contract
    hub_contract.update_config(None, Some(FEE_COLLECTOR.to_string()), None)?;

    Ok(())

}
