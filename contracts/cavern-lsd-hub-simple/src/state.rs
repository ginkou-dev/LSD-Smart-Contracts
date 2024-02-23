
use cw_storage_plus::Item;

use basset::hub::{Config, State, Parameters};

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const STATE: Item<State> = Item::new("\u{0}\u{5}state");
