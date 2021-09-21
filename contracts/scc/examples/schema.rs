use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use scc::msg::*;
use scc::state::{Config, State};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(GetStateResponse), &out_dir);
    export_schema(&schema_for!(GetConfigResponse), &out_dir);
    export_schema(&schema_for!(GetStrategyInfoResponse), &out_dir);
    export_schema(&schema_for!(GetUserRewardInfo), &out_dir);
    export_schema(&schema_for!(GetStrategiesListResponse), &out_dir);
    export_schema(&schema_for!(GetAllStrategiesResponse), &out_dir);
    export_schema(&schema_for!(GetUserResponse), &out_dir);
}
