use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub type SocialInfo = (Platform, ProfileId);
pub type Platform = String;
pub type ProfileId = String;
pub const SOCIAL_TO_ADDRESS: Map<SocialInfo, Addr> = Map::new("social_to_address");
pub const ADDRESS_TO_SOCIAL: Map<Addr, SocialInfo> = Map::new("wallet_to_social");
pub const OWNER: Item<Addr> = Item::new("owner");
pub const ACCEPTED_TOKEN: Item<Vec<String>> = Item::new("accepted_token");

