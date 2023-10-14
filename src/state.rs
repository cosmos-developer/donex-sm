use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
pub type SocialInfo = (Platform, ProfileId);
pub type Platform = String;
pub type ProfileId = String;
pub struct InfoIndexes<'a> {
    pub address: MultiIndex<'a, Addr, UserInfo, String>,
    pub social_info: MultiIndex<'a, (String, String), UserInfo, String>,
}
impl<'a> IndexList<UserInfo> for InfoIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<UserInfo>> + '_> {
        let v: Vec<&dyn Index<UserInfo>> = vec![&self.address, &self.social_info];
        Box::new(v.into_iter())
    }
}
pub const fn infos<'a>() -> IndexedMap<'a, &'a str, UserInfo, InfoIndexes<'a>> {
    let indexes = InfoIndexes {
        address: MultiIndex::new(
            |_pk: &[u8], d: &UserInfo| d.address.clone(),
            "infos",
            "infos__address",
        ),
        social_info: MultiIndex::new(
            |_pk: &[u8], d: &UserInfo| (d.platform_id.clone(), d.profile_id.clone()),
            "infos",
            "infos__social_info",
        ),
    };
    IndexedMap::new("infos", indexes)
}
#[cw_serde]
pub struct UserInfo {
    pub address: Addr,
    pub platform_id: String,
    pub profile_id: String,
}
pub const USER_INFOS: IndexedMap<&str, UserInfo, InfoIndexes> = infos();

pub const OWNER: Item<Addr> = Item::new("owner");
pub const ACCEPTED_TOKEN: Item<Vec<String>> = Item::new("accepted_token");
