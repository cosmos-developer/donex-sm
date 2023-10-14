#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SocialResponse};
use crate::state::{Platform, ProfileId, SocialInfo, UserInfo, ACCEPTED_TOKEN, OWNER, USER_INFOS};
use cw20_base::contract::{execute_send, execute_transfer};
const CONTRACT_NAME: &str = "cosmos:donex-sm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    OWNER.save(deps.storage, &info.sender)?;
    ACCEPTED_TOKEN.save(deps.storage, &msg.accepted_token)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;
    match msg {
        SubmitSocial {
            social_info,
            address,
        } => submit_social_link(deps, info, social_info, address),
        Donate { recipient, amount } => donate(deps, env, info, recipient, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetSocial {
            profile_id,
            platform,
        } => to_binary(&query_by_social_link(deps, profile_id, platform)?),
    }
}
// submit link between social platform accounts and chain address
pub fn submit_social_link(
    deps: DepsMut,
    info: MessageInfo,
    social_info: SocialInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    // Check authorization
    if info.sender != OWNER.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }
    let user_info = UserInfo {
        address: address.clone(),
        platform_id: social_info.0,
        profile_id: social_info.1,
    };
    USER_INFOS.save(deps.storage, address.as_ref(), &user_info)?;

    Ok(Response::new().add_attribute("method", "submit_social_link"))
}

pub fn donate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    const FEE_RATIO: u64 = 5;
    let accepted_token = ACCEPTED_TOKEN.load(deps.storage)?;
    let _owner = OWNER.load(deps.storage);
    let denom = accepted_token.first().unwrap();
    let donation = cw_utils::must_pay(&info, denom)?.u128();

    // Calculate fee and actual amount receive
    let fee = donation / FEE_RATIO as u128 * 100;
    let actual = donation - fee;

    execute_transfer(
        deps.branch(),
        env.clone(),
        info.clone(),
        recipient.to_string(),
        actual.into(),
    )?;
    let msg = format!("Handling donation from {} to {}", info.sender, recipient);
    execute_send(
        deps.branch(),
        env.clone(),
        info.clone(),
        env.contract.address.to_string(),
        amount,
        cosmwasm_std::Binary(msg.into_bytes()),
    )?;
    Ok(Response::new())
}
fn query_by_social_link(
    deps: Deps,
    profile_id: ProfileId,
    platform: Platform,
) -> StdResult<SocialResponse> {
    let social_info = (platform, profile_id);
    // Query by (platform, profile_id)
    let user_infos: Vec<_> = USER_INFOS
        .idx
        .social_info
        .prefix(social_info)
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    if user_infos.is_empty() {
        return Ok(SocialResponse { address: vec![] });
    }
    let addresses = user_infos
        .iter()
        .map(|user_info| user_info.1.address.clone())
        .collect::<Vec<_>>();
    Ok(SocialResponse { address: addresses })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};
    #[test]
    fn submit_social() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    accepted_token: vec!["ATOM".to_string()],
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();
        let resp = app
            .execute_contract(
                Addr::unchecked("owner"),
                addr,
                &ExecuteMsg::SubmitSocial {
                    social_info: ("twitter".to_string(), "123".to_string()),
                    address: Addr::unchecked("abc"),
                },
                &[],
            )
            .unwrap();
        let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "method")
                .unwrap()
                .value,
            "submit_social_link"
        );
    }

    #[test]
    fn social_query() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            InstantiateMsg {
                accepted_token: vec!["ATOM".to_string()],
            },
        )
        .unwrap();
        let resp = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::GetSocial {
                profile_id: "1".to_string(),
                platform: "twitter".to_string(),
            },
        );
        assert!(resp.is_ok());
        let resp: SocialResponse = from_binary(&resp.unwrap()).unwrap();

        assert_eq!(resp, SocialResponse { address: vec![] });
        // execute
        let resp = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            ExecuteMsg::SubmitSocial {
                social_info: ("twitter".to_string(), "123".to_string()),
                address: Addr::unchecked("abc"),
            },
        );
        assert!(resp.is_ok());
        // query again
        let resp = query(
            deps.as_ref(),
            env,
            QueryMsg::GetSocial {
                platform: "twitter".to_string(),
                profile_id: "123".to_string(),
            },
        )
        .unwrap();
        let resp: SocialResponse = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            SocialResponse {
                address: vec![Addr::unchecked("abc")]
            }
        );
    }
}
