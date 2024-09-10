use crate::{
    error::ContractError,
    math::{mul_ratio_u128, sub_u128},
    msg::{BalanceChangeEvent, BalanceChangeMsg},
    state::{ACCOUNT_STATS, BASE_TOKEN, CURVE},
};
use cosmwasm_std::{attr, ensure_eq, Addr, Response, Storage, Uint128};

use super::Context;

pub fn exec_on_balance_change(
    ctx: Context,
    msg: BalanceChangeMsg,
) -> Result<Response, ContractError> {
    let Context { deps, info, .. } = ctx;
    let curve = CURVE.load(deps.storage)?;
    let quote_decimals = curve.quote_decimals;
    let event = msg.event;

    ensure_eq!(
        info.sender,
        BASE_TOKEN.load(deps.storage)?.get_address().unwrap(),
        ContractError::NotAuthorized {
            reason: format!("OnBalanceChange received msg from unrecognized cw20: {}", info.sender)
        }
    );

    match event {
        BalanceChangeEvent::Transfer {
            initiator,
            initiator_balance,
            recipient,
            amount: delta,
            ..
        } => {
            update_initiator_total_cost(deps.storage, quote_decimals, &initiator, initiator_balance, delta)?;
            update_recipient_total_cost(deps.storage, &recipient, delta)?;
        },
        BalanceChangeEvent::Burn {
            initiator,
            initiator_balance,
            amount: delta,
            ..
        } => {
            update_initiator_total_cost(deps.storage, quote_decimals, &initiator, initiator_balance, delta)?;
        },
        BalanceChangeEvent::Mint {
            recipient,
            amount: delta,
            ..
        } => {
            update_recipient_total_cost(deps.storage, &recipient, delta)?;
        },
    }

    Ok(Response::new().add_attributes(vec![attr("action", "on_balance_change")]))
}

pub fn calc_avg_cost_basis(
    quote_decimals: u8,
    total_cost: Uint128,
    total_base_amount: Uint128,
) -> Result<Uint128, ContractError> {
    mul_ratio_u128(total_cost, 10u128.pow(quote_decimals as u32), total_base_amount)
}

pub fn update_initiator_total_cost(
    store: &mut dyn Storage,
    quote_decimals: u8,
    initiator: &Addr,
    initiator_balance: Uint128,
    delta: Uint128,
) -> Result<(), ContractError> {
    ACCOUNT_STATS.update(store, initiator, |maybe_stats| -> Result<_, ContractError> {
        let mut initator_stats = maybe_stats.unwrap_or_default();
        if delta.is_zero() {
            initator_stats.total_cost = Uint128::zero();
        } else {
            let cost_basis = calc_avg_cost_basis(quote_decimals, initator_stats.total_cost, initiator_balance)?;
            let cost_of_tokens_sold = mul_ratio_u128(delta, cost_basis, 10u128.pow(quote_decimals as u32))?;
            initator_stats.total_cost = sub_u128(initator_stats.total_cost, cost_of_tokens_sold)?;
        }
        Ok(initator_stats)
    })?;
    Ok(())
}

pub fn update_recipient_total_cost(
    store: &mut dyn Storage,
    recipient: &Addr,
    delta: Uint128,
) -> Result<(), ContractError> {
    ACCOUNT_STATS.update(store, &recipient, |maybe_stats| -> Result<_, ContractError> {
        let mut recipient_stats = maybe_stats.unwrap_or_default();
        recipient_stats.total_cost = sub_u128(recipient_stats.total_cost, delta)?;
        Ok(recipient_stats)
    })?;
    Ok(())
}
