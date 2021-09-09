use cosmwasm_std::{Coin, Decimal};
use stader_utils::coin_utils::{
    check_equal_deccoin_vector, deccoin_vec_to_coin_vec, merge_dec_coin_vector,
    multiply_deccoin_vector_with_decimal, DecCoin, DecCoinVecOp, Operation,
};

pub fn get_user_airdrops(
    global_airdrop_pointer: &Vec<DecCoin>,
    user_airdrop_pointer: &Vec<DecCoin>,
    user_shares: Decimal,
) -> Option<Vec<Coin>> {
    if global_airdrop_pointer.is_empty() {
        return None;
    }

    if check_equal_deccoin_vector(global_airdrop_pointer, user_airdrop_pointer) {
        return None;
    }

    let airdrop_pointer_difference = merge_dec_coin_vector(
        &global_airdrop_pointer,
        DecCoinVecOp {
            fund: user_airdrop_pointer.clone(),
            operation: Operation::Sub,
        },
    );

    let user_airdrops =
        multiply_deccoin_vector_with_decimal(&airdrop_pointer_difference, user_shares);

    Some(deccoin_vec_to_coin_vec(&user_airdrops))
}
