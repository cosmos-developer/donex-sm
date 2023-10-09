use crate::state::{Platform, ProfileId, SocialInfo};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SubmitSocial {
        social_info: SocialInfo,
        address: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SocialResponse)]
    GetSocial {
        profile_id: ProfileId,
        platform: Platform,
    },
}
#[cw_serde]
pub struct SocialResponse {
    pub address: Addr,
}
