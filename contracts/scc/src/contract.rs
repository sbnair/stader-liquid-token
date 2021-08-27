#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Attribute, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Timestamp, Uint128, WasmMsg,
};

use crate::error::ContractError;
use crate::helpers::{get_sic_total_tokens, get_user_strategy_data, strategy_supports_airdrops};
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse, UpdateUserAirdropsRequest,
    UpdateUserRewardsRequest,
};
use crate::state::{
    State, StrategyInfo, StrategyMetadata, UserRewardInfo, UserStrategyInfo, STATE,
    STRATEGY_INFO_MAP, STRATEGY_METADATA_MAP, USER_REWARD_INFO_MAP,
};
use crate::user::get_user_airdrops;
use crate::utils::{
    decimal_division_in_256, decimal_multiplication_in_256, decimal_summation_in_256,
    merge_coin_vector, CoinVecOp, Operation,
};
use sic_base::msg::{ExecuteMsg as sic_execute_msg, QueryMsg as sic_query_msg};
use std::borrow::Borrow;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        manager: info.sender.clone(),
        scc_denom: msg.strategy_denom,
        contract_genesis_block_height: _env.block.height,
        contract_genesis_timestamp: _env.block.time,
        total_accumulated_rewards: Uint128::zero(),
        current_rewards_in_scc: Uint128::zero(),
        total_accumulated_airdrops: vec![],
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterStrategy {
            strategy_id,
            unbonding_period,
            sic_contract_address,
            supported_airdrops,
        } => try_register_strategy(
            deps,
            _env,
            info,
            strategy_id,
            unbonding_period,
            sic_contract_address,
            supported_airdrops,
        ),
        ExecuteMsg::DeactivateStrategy { strategy_id } => {
            try_deactivate_strategy(deps, _env, info, strategy_id)
        }
        ExecuteMsg::ActivateStrategy { strategy_id } => {
            try_activate_strategy(deps, _env, info, strategy_id)
        }
        ExecuteMsg::RemoveStrategy { strategy_id } => {
            try_remove_strategy(deps, _env, info, strategy_id)
        }
        ExecuteMsg::UpdateUserRewards {
            update_user_rewards_requests,
        } => try_update_user_rewards(deps, _env, info, update_user_rewards_requests),
        ExecuteMsg::UpdateUserAirdrops {
            update_user_airdrops_requests,
        } => try_update_user_airdrops(deps, _env, info, update_user_airdrops_requests),
        ExecuteMsg::UndelegateRewards {
            amount,
            strategy_id,
        } => try_undelegate_rewards(deps, _env, info, amount, strategy_id),
        ExecuteMsg::ClaimAirdrops { strategy_id } => {
            try_claim_airdrops(deps, _env, info, strategy_id)
        }
        ExecuteMsg::WithdrawRewards {
            undelegation_timestamp,
            strategy_id,
        } => try_withdraw_rewards(deps, _env, info, undelegation_timestamp, strategy_id),
        ExecuteMsg::WithdrawAirdrops {} => try_withdraw_airdrops(deps, _env, info),
    }
}

pub fn try_undelegate_rewards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
    strategy_id: String,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn try_claim_airdrops(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    strategy_id: String,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn try_withdraw_rewards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    undelegation_timestamp: Timestamp,
    strategy_id: String,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn try_withdraw_airdrops(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn try_register_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    strategy_id: String,
    unbonding_period: Option<u64>,
    sic_contract_address: Addr,
    supported_airdrops: Vec<String>,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage).unwrap();
    if info.sender != state.manager {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(_) = STRATEGY_INFO_MAP
        .may_load(deps.storage, strategy_id.clone())
        .unwrap()
    {
        return Err(ContractError::StrategyInfoAlreadyExists {});
    }

    if let Some(_) = STRATEGY_METADATA_MAP
        .may_load(deps.storage, strategy_id.clone())
        .unwrap()
    {
        return Err(ContractError::StrategyMetadataAlreadyExists {});
    }

    STRATEGY_INFO_MAP.save(
        deps.storage,
        strategy_id.clone(),
        &StrategyInfo {
            name: strategy_id.clone().to_string(),
            sic_contract_address,
            unbonding_period,
            supported_airdrops,
            is_active: false,
        },
    )?;
    STRATEGY_METADATA_MAP.save(
        deps.storage,
        strategy_id.clone(),
        &StrategyMetadata {
            name: strategy_id.to_string(),
            total_shares: Decimal::zero(),
            global_airdrop_pointer: vec![],
            total_airdrops_accumulated: vec![],
            shares_per_token_ratio: Decimal::zero(),
            current_unprocessed_undelegations: Uint128::zero(),
        },
    )?;

    Ok(Response::default())
}

pub fn try_deactivate_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    strategy_id: String,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage).unwrap();
    if info.sender != state.manager {
        return Err(ContractError::Unauthorized {});
    }

    STRATEGY_INFO_MAP.update(
        deps.storage,
        strategy_id,
        |strategy_info_option| -> Result<_, ContractError> {
            if strategy_info_option.is_none() {
                return Err(ContractError::StrategyInfoDoesNotExist {});
            }

            let mut strategy_info = strategy_info_option.unwrap();
            strategy_info.is_active = false;
            Ok(strategy_info)
        },
    )?;

    Ok(Response::default())
}

pub fn try_activate_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    strategy_id: String,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage).unwrap();
    if info.sender != state.manager {
        return Err(ContractError::Unauthorized {});
    }

    STRATEGY_INFO_MAP.update(
        deps.storage,
        strategy_id,
        |strategy_info_option| -> Result<_, ContractError> {
            if strategy_info_option.is_none() {
                return Err(ContractError::StrategyInfoDoesNotExist {});
            }

            let mut strategy_info = strategy_info_option.unwrap();
            strategy_info.is_active = false;
            Ok(strategy_info)
        },
    )?;

    Ok(Response::default())
}

pub fn try_remove_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    strategy_id: String,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage).unwrap();
    if info.sender != state.manager {
        return Err(ContractError::Unauthorized {});
    }

    STRATEGY_INFO_MAP.remove(deps.storage, strategy_id);

    Ok(Response::default())
}

pub fn try_update_user_rewards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    update_user_rewards_requests: Vec<UpdateUserRewardsRequest>,
) -> Result<Response, ContractError> {
    // check for manager?
    let state = STATE.load(deps.storage).unwrap();
    if info.sender != state.manager {
        return Err(ContractError::Unauthorized {});
    }

    if update_user_rewards_requests.is_empty() {
        return Ok(Response::default());
    }

    let mut messages: Vec<WasmMsg> = vec![];
    let mut logs: Vec<Attribute> = vec![];
    // iterate thru all requests
    for user_request in update_user_rewards_requests {
        let user_strategy = user_request.strategy_id;
        let user_amount = user_request.rewards;
        let user_addr = user_request.user;

        let mut strategy_info: StrategyInfo = StrategyInfo::default();
        if let Some(strategy_info_mapping) = STRATEGY_INFO_MAP
            .may_load(deps.storage, user_strategy.clone())
            .unwrap()
        {
            strategy_info = strategy_info_mapping;
        } else {
            // TODO: bchain99 - log something out here
            continue;
        }

        let mut strategy_metadata: StrategyMetadata = StrategyMetadata::default();
        if let Some(strategy_metadata_mapping) = STRATEGY_METADATA_MAP
            .may_load(deps.storage, user_strategy.clone())
            .unwrap()
        {
            strategy_metadata = strategy_metadata_mapping;
        } else {
            // TODO: bchain99 - log something out here
            continue;
        }

        // fetch the total tokens from the SIC contract and update the S/T ratio for the strategy
        let total_tokens = get_sic_total_tokens(deps.querier, &strategy_info.sic_contract_address)
            .total_tokens
            .unwrap_or_else(|| Uint128::zero());
        let mut shares_per_token_ratio = Decimal::one();
        if !total_tokens.is_zero() {
            shares_per_token_ratio = decimal_division_in_256(
                strategy_metadata.total_shares,
                Decimal::from_ratio(total_tokens, 1_u128),
            );
        }
        strategy_metadata.shares_per_token_ratio = shares_per_token_ratio;

        let mut user_reward_info = UserRewardInfo::new();
        if let Some(user_reward_info_mapping) = USER_REWARD_INFO_MAP
            .may_load(deps.storage, &user_addr)
            .unwrap()
        {
            user_reward_info = user_reward_info_mapping;
        }

        // update the user airdrop pointer and allocate the user pending airdrops for each strategy
        let mut user_strategy_data: UserStrategyInfo;
        if let Some(user_strategy_data_mapping) =
            get_user_strategy_data(&user_reward_info, user_strategy.clone())
        {
            user_strategy_data = user_strategy_data_mapping;
        } else {
            user_strategy_data = UserStrategyInfo::new(user_strategy.clone());
            if strategy_supports_airdrops(&strategy_info) {
                user_strategy_data.airdrop_pointer =
                    strategy_metadata.global_airdrop_pointer.clone();
            }
        }

        // update user shares based on the S/T ratio
        let user_shares = decimal_multiplication_in_256(
            shares_per_token_ratio,
            Decimal::from_ratio(user_amount, 1_u128),
        );

        user_strategy_data.shares =
            decimal_summation_in_256(user_strategy_data.shares, user_shares);

        // update total strategy shares by adding up the user_shares
        strategy_metadata.total_shares =
            decimal_summation_in_256(strategy_metadata.total_shares, user_shares);

        // do statewise book-keeping like adding up accumulated_rewards
        STATE.update(deps.storage, |mut state| -> StdResult<_> {
            state.total_accumulated_rewards = state
                .total_accumulated_rewards
                .checked_add(user_amount)
                .unwrap();
            Ok(state)
        });

        // send the rewards to sic
        messages.push(WasmMsg::Execute {
            contract_addr: String::from(strategy_info.sic_contract_address),
            msg: to_binary(&sic_execute_msg::TransferRewards {}).unwrap(),
            funds: vec![Coin::new(user_amount.u128(), state.scc_denom.clone())],
        });

        // save up the states
        STRATEGY_METADATA_MAP.save(deps.storage, user_strategy.clone(), &strategy_metadata);

        user_reward_info.strategies.push(user_strategy_data);
        USER_REWARD_INFO_MAP.save(deps.storage, &user_addr, &user_reward_info);
    }

    Ok(Response::new().add_messages(messages))
}

// This assumes that the validator contract will transfer ownership of the airdrops
// from the validator contract to the SCC contract.
pub fn try_update_user_airdrops(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    update_user_airdrops_requests: Vec<UpdateUserAirdropsRequest>,
) -> Result<Response, ContractError> {
    // check for manager?
    let state = STATE.load(deps.storage).unwrap();
    if info.sender != state.manager {
        return Err(ContractError::Unauthorized {});
    }

    if update_user_airdrops_requests.is_empty() {
        return Ok(Response::default());
    }

    // iterate thru update_user_airdrops_request
    let mut total_scc_airdrops: Vec<Coin> = state.total_accumulated_airdrops;
    // accumulate the airdrops in the SCC state.
    for user_request in update_user_airdrops_requests {
        let user = user_request.user;
        let user_airdrops = user_request.pool_airdrops;

        total_scc_airdrops = merge_coin_vector(
            total_scc_airdrops.clone(),
            CoinVecOp {
                fund: user_airdrops.clone(),
                operation: Operation::Add,
            },
        );

        // fetch the user rewards info
        let mut user_reward_info = UserRewardInfo::new();
        if let Some(user_reward_info_mapping) =
            USER_REWARD_INFO_MAP.may_load(deps.storage, &user).unwrap()
        {
            user_reward_info = user_reward_info_mapping;
        }

        user_reward_info.pending_airdrops = merge_coin_vector(
            user_reward_info.pending_airdrops,
            CoinVecOp {
                fund: user_airdrops,
                operation: Operation::Add,
            },
        );

        USER_REWARD_INFO_MAP.save(deps.storage, &user, &user_reward_info);
    }

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.total_accumulated_airdrops = total_scc_airdrops;
        Ok(state)
    });

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query_state(deps)?),
    }
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        state: Option::from(state),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::check_equal_vec;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            strategy_denom: "uluna".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let env = mock_env();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query_state(deps.as_ref()).unwrap();
        assert_eq!(
            res.state.unwrap(),
            State {
                manager: info.sender,
                scc_denom: "uluna".to_string(),
                contract_genesis_block_height: env.block.height,
                contract_genesis_timestamp: env.block.time,
                total_accumulated_rewards: Uint128::zero(),
                current_rewards_in_scc: Uint128::zero(),
                total_accumulated_airdrops: vec![]
            }
        );
    }

    #[test]
    fn test__try_update_user_airdrops_fail() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            strategy_denom: "uluna".to_string(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        /*
           Test - 1. Unauthorized
        */
        let mut err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("not-creator", &[]),
            ExecuteMsg::UpdateUserAirdrops {
                update_user_airdrops_requests: vec![],
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));

        /*
           Test - 2. Empty request object
        */
        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("creator", &[]),
            ExecuteMsg::UpdateUserAirdrops {
                update_user_airdrops_requests: vec![],
            },
        )
        .unwrap();
        assert_eq!(res, Response::default());
    }

    #[test]
    fn test__try_update_user_airdrops_success() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            strategy_denom: "uluna".to_string(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let user1 = Addr::unchecked("user-1");
        let user2 = Addr::unchecked("user-2");
        let user3 = Addr::unchecked("user-3");
        let user4 = Addr::unchecked("user-4");

        /*
           Test - 1. First airdrops
        */
        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("creator", &[]),
            ExecuteMsg::UpdateUserAirdrops {
                update_user_airdrops_requests: vec![
                    UpdateUserAirdropsRequest {
                        user: user1.clone(),
                        pool_airdrops: vec![Coin::new(100_u128, "abc"), Coin::new(50_u128, "def")],
                    },
                    UpdateUserAirdropsRequest {
                        user: user2.clone(),
                        pool_airdrops: vec![Coin::new(50_u128, "abc"), Coin::new(50_u128, "def")],
                    },
                    UpdateUserAirdropsRequest {
                        user: user3.clone(),
                        pool_airdrops: vec![Coin::new(200_u128, "abc"), Coin::new(100_u128, "def")],
                    },
                    UpdateUserAirdropsRequest {
                        user: user4.clone(),
                        pool_airdrops: vec![],
                    },
                ],
            },
        )
        .unwrap();
        let state_response = query_state(deps.as_ref()).unwrap();
        assert_ne!(state_response.state, None);
        let state = state_response.state.unwrap();
        assert!(check_equal_vec(
            state.total_accumulated_airdrops,
            vec![Coin::new(350_u128, "abc"), Coin::new(200_u128, "def")]
        ));
        let user_reward_info_1_opt = USER_REWARD_INFO_MAP
            .may_load(deps.as_mut().storage, &user1)
            .unwrap();
        assert_ne!(user_reward_info_1_opt, None);
        let user_reward_info_1 = user_reward_info_1_opt.unwrap();
        assert!(check_equal_vec(
            user_reward_info_1.pending_pool_airdrops,
            vec![Coin::new(100_u128, "abc"), Coin::new(50_u128, "def")]
        ));
        let user_reward_info_2_opt = USER_REWARD_INFO_MAP
            .may_load(deps.as_mut().storage, &user2)
            .unwrap();
        assert_ne!(user_reward_info_2_opt, None);
        let user_reward_info_2 = user_reward_info_2_opt.unwrap();
        assert!(check_equal_vec(
            user_reward_info_2.pending_pool_airdrops,
            vec![Coin::new(50_u128, "abc"), Coin::new(50_u128, "def")]
        ));
        let user_reward_info_3_opt = USER_REWARD_INFO_MAP
            .may_load(deps.as_mut().storage, &user3)
            .unwrap();
        assert_ne!(user_reward_info_3_opt, None);
        let user_reward_info_3 = user_reward_info_3_opt.unwrap();
        assert!(check_equal_vec(
            user_reward_info_3.pending_pool_airdrops,
            vec![Coin::new(200_u128, "abc"), Coin::new(100_u128, "def")]
        ));
        let user_reward_info_4_opt = USER_REWARD_INFO_MAP
            .may_load(deps.as_mut().storage, &user4)
            .unwrap();
        assert_ne!(user_reward_info_4_opt, None);
        let user_reward_info_4 = user_reward_info_4_opt.unwrap();
        assert!(check_equal_vec(
            user_reward_info_4.pending_pool_airdrops,
            vec![]
        ));

        /*
           Test - 2. updating the user airdrops with existing user_airdrops
        */
        STATE.update(
            deps.as_mut().storage,
            |mut state| -> Result<_, ContractError> {
                state.total_accumulated_airdrops =
                    vec![Coin::new(100_u128, "abc"), Coin::new(200_u128, "def")];
                Ok(state)
            },
        );

        USER_REWARD_INFO_MAP.save(
            deps.as_mut().storage,
            &user1,
            &UserRewardInfo {
                strategies: vec![],
                pending_airdrops: vec![Coin::new(10_u128, "abc"), Coin::new(200_u128, "def")],
            },
        );
        USER_REWARD_INFO_MAP.save(
            deps.as_mut().storage,
            &user2,
            &UserRewardInfo {
                strategies: vec![],
                pending_airdrops: vec![Coin::new(20_u128, "abc"), Coin::new(100_u128, "def")],
            },
        );
        USER_REWARD_INFO_MAP.save(
            deps.as_mut().storage,
            &user3,
            &UserRewardInfo {
                strategies: vec![],
                pending_airdrops: vec![Coin::new(30_u128, "abc"), Coin::new(50_u128, "def")],
            },
        );
        USER_REWARD_INFO_MAP.save(
            deps.as_mut().storage,
            &user4,
            &UserRewardInfo {
                strategies: vec![],
                pending_airdrops: vec![Coin::new(40_u128, "abc"), Coin::new(80_u128, "def")],
            },
        );

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("creator", &[]),
            ExecuteMsg::UpdateUserAirdrops {
                update_user_airdrops_requests: vec![
                    UpdateUserAirdropsRequest {
                        user: user1.clone(),
                        pool_airdrops: vec![Coin::new(100_u128, "abc"), Coin::new(50_u128, "def")],
                    },
                    UpdateUserAirdropsRequest {
                        user: user2.clone(),
                        pool_airdrops: vec![Coin::new(50_u128, "abc"), Coin::new(50_u128, "def")],
                    },
                    UpdateUserAirdropsRequest {
                        user: user3.clone(),
                        pool_airdrops: vec![Coin::new(200_u128, "abc"), Coin::new(100_u128, "def")],
                    },
                    UpdateUserAirdropsRequest {
                        user: user4.clone(),
                        pool_airdrops: vec![],
                    },
                ],
            },
        )
        .unwrap();
        let state_response = query_state(deps.as_ref()).unwrap();
        assert_ne!(state_response.state, None);
        let state = state_response.state.unwrap();
        assert!(check_equal_vec(
            state.total_accumulated_airdrops,
            vec![Coin::new(450_u128, "abc"), Coin::new(400_u128, "def")]
        ));
        let user_reward_info_1_opt = USER_REWARD_INFO_MAP
            .may_load(deps.as_mut().storage, &user1)
            .unwrap();
        assert_ne!(user_reward_info_1_opt, None);
        let user_reward_info_1 = user_reward_info_1_opt.unwrap();
        assert!(check_equal_vec(
            user_reward_info_1.pending_pool_airdrops,
            vec![Coin::new(110_u128, "abc"), Coin::new(250_u128, "def")]
        ));
        let user_reward_info_2_opt = USER_REWARD_INFO_MAP
            .may_load(deps.as_mut().storage, &user2)
            .unwrap();
        assert_ne!(user_reward_info_2_opt, None);
        let user_reward_info_2 = user_reward_info_2_opt.unwrap();
        assert!(check_equal_vec(
            user_reward_info_2.pending_pool_airdrops,
            vec![Coin::new(70_u128, "abc"), Coin::new(150_u128, "def")]
        ));
        let user_reward_info_3_opt = USER_REWARD_INFO_MAP
            .may_load(deps.as_mut().storage, &user3)
            .unwrap();
        assert_ne!(user_reward_info_3_opt, None);
        let user_reward_info_3 = user_reward_info_3_opt.unwrap();
        assert!(check_equal_vec(
            user_reward_info_3.pending_pool_airdrops,
            vec![Coin::new(230_u128, "abc"), Coin::new(150_u128, "def")]
        ));
        let user_reward_info_4_opt = USER_REWARD_INFO_MAP
            .may_load(deps.as_mut().storage, &user4)
            .unwrap();
        assert_ne!(user_reward_info_4_opt, None);
        let user_reward_info_4 = user_reward_info_4_opt.unwrap();
        assert!(check_equal_vec(
            user_reward_info_4.pending_pool_airdrops,
            vec![Coin::new(40_u128, "abc"), Coin::new(80_u128, "def")]
        ));
    }
}
