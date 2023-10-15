use crate::state::{Platform, ProfileId, SocialInfo};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
#[cw_serde]
pub struct InstantiateMsg {
    pub accepted_token: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    SubmitSocial {
        social_info: SocialInfo,
        address: Addr,
    },
    Donate {
        recipient: Addr,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetAddressesBySocialResponse)]
    GetAddressesBySocial {
        profile_id: ProfileId,
        platform: Platform,
    },
    #[returns(GetSocialsByAddressResponse)]
    GetSocialsByAddress { address: Addr },
}
#[cw_serde]
pub struct GetAddressesBySocialResponse {
    pub address: Vec<Addr>,
}
#[cw_serde]
pub struct GetSocialsByAddressResponse {
    pub social_infos: Vec<(Platform, ProfileId)>,
}
