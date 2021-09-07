use crate::state::State;
use cosmwasm_std::{Addr, Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub scc_address: Addr,
    // denomination of the staking coin
    pub vault_denom: String,
    // initial set of validators who make up the validator pool
    pub initial_validators: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    TransferRewards {},
    UndelegateRewards {
        amount: Uint128,
    },
    TransferUndelegatedRewards {
        amount: Uint128,
    },
    Swap {},
    Reinvest {},
    RedeemRewards {},
    // Called by the manager to claim airdrops from different protocols. Airdrop token contract fed from SCC.
    // The ownership of the airdrops is transferred back to the SCC.
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
    GetState {},
    GetUndelegatedFunds { amount: Uint128 },
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
pub struct GetUndelegatedFunds {
    pub undelegated_funds: Option<Uint128>,
}
