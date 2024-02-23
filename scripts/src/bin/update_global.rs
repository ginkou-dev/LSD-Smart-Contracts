use cw_orch::{prelude::DaemonBuilder, tokio::runtime::Runtime};
use scripts::{hub_reward_migration::{migrate_migaloo_swap::upload_reward, reward_addr_and_migrate_hub::upload_hub_simple}, MIGALOO_1};


fn update_global() -> anyhow::Result<()>{

    dotenv::dotenv()?;
    pretty_env_logger::init();

    // We upload code_ids
    let rt = Runtime::new()?;
    let app = DaemonBuilder::default()
        .chain(MIGALOO_1)
        .handle(rt.handle())
        .build()?;

    upload_reward(app.clone())?;
    upload_hub_simple(app.clone())?;

    // Other functions have to be executed inside the multisig

    Ok(())
}


fn main(){
    update_global().unwrap()
}