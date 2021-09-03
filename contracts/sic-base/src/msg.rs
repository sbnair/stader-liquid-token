use crate::state::{BatchUndelegationRecord, State};
use cosmwasm_std::{Addr, Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub scc_address: Addr,
    // denomination of the staking coin
    pub strategy_denom: String,
    // unbonding period in seconds (defaults to 21 days + 3600s)
    pub unbonding_period: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // called by SCC to move rewards to SIC from SCC
    TransferRewards {},
    // called by SCC to undelegate 'amount' from SIC strategy back to SIC contract
    // All the batching for the undelegations is handled in the SCC
    UndelegateRewards {
        amount: Uint128,
    },
    // Called by the SCC to finally withdraw rewards from SIC after the unbonding period
    WithdrawRewards {
        user: Addr,
        amount: Uint128,
        undelegation_batch_id: u64,
    },
    // Called by the SCC to claim airdrops from different protocols for the strategy (if airdrop applies)
    // Airdrop token contract is fed from SCC.
    // The airdrops are claimed by the SIC contract and then the ownership of the airdrops are transferred back to the SCC.
    ClaimAirdrops {
        airdrop_token_contract: Addr,
        // used to transfer ownership from SIC to SCC
        cw20_token_contract: Addr,
        airdrop_token: String,
        amount: Uint128,
        claim_msg: Binary,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetTotalTokens {},
    GetCurrentUndelegationBatchId {},
    GetUndelegationBatchInfo { undelegation_batch_id: u64 },
    GetState {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetStateResponse {
    pub state: Option<State>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetTotalTokensResponse {
    pub total_tokens: Option<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetCurrentUndelegationBatchIdResponse {
    pub current_undelegation_batch_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetUndelegationBatchInfoResponse {
    pub undelegation_batch_info: Option<BatchUndelegationRecord>,
}
