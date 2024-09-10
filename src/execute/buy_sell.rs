use cosmwasm_std::{attr, Response, Storage, SubMsg, Uint128};

use crate::{
    error::ContractError,
    math::{add_u128, add_u32, add_u64, mul_pct_u128, sub_u128},
    models::{account::MaxSwapInfo, ohlc::OhlcBar},
    msg::{BuyMsg, SellMsg},
    state::{
        ACCOUNT_STATS, BASE_TOKEN, CURVE, FEE_ADDR, FEE_PCT_BUY, FEE_PCT_SELL, MAKER_STATS, NET_MAKER_FEE,
        NET_TAKER_FEE, QUOTE_TOKEN, TAKER_STATS,
    },
    token::Token,
    utils::resolve_swap_initiator,
};

use super::Context;

pub fn exec_buy(
    ctx: Context,
    msg: BuyMsg,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    let BuyMsg {
        initiator,
        min_out_amount,
    } = msg;

    let mut curve = CURVE.load(deps.storage)?;
    let quote_token = QUOTE_TOKEN.load(deps.storage)?;
    let base_token = BASE_TOKEN.load(deps.storage)?;

    // Amount we're trying to swap in. If ammount is None, it implies that the
    // quote token is a native coin in info.funds; otherwise, it's a CW20
    let in_amount_pre_fee = if let Some(amount) = amount {
        amount
    } else {
        if let Some(coin) = quote_token.find_in_funds(&info.funds, None) {
            coin.amount
        } else {
            return Err(ContractError::MissingFunds {
                denom: quote_token.get_denom().unwrap(),
            });
        }
    };

    // Compute buy or sell-side platform fee
    let fee_amount = mul_pct_u128(in_amount_pre_fee, FEE_PCT_BUY.load(deps.storage)?)?;

    // Increment total historical aggregate fee amount
    NET_TAKER_FEE.update(deps.storage, |n| -> Result<_, ContractError> {
        add_u128(n, fee_amount)
    })?;

    // Subtract fee from amount recieved by sender
    let in_amount = sub_u128(in_amount_pre_fee, fee_amount)?;

    // Perform AMM swap
    let out_amount = curve.buy(in_amount, min_out_amount)?;

    // Get initiator. The initiator is either the user performing the tx or the
    // user on whose behalf the operator is performing it.
    let initiator = resolve_swap_initiator(deps.storage, deps.api, &info.sender, amount.is_some(), initiator, "buy")?;

    // Update initiator's account info
    ACCOUNT_STATS.update(deps.storage, &initiator, |maybe_stats| -> Result<_, ContractError> {
        let mut stats = maybe_stats.unwrap_or_default();
        stats.n_buys = add_u32(stats.n_buys, 1)?;
        stats.net_quote_in = add_u128(stats.net_quote_in, in_amount)?;
        stats.net_base_out = add_u128(stats.net_base_out, out_amount)?;
        Ok(stats)
    })?;

    // Update global stats
    TAKER_STATS.update(deps.storage, |mut stats| -> Result<_, ContractError> {
        stats.n = add_u64(stats.n, 1u64)?;
        if stats.max.is_none() || stats.max.clone().and_then(|m| Some(out_amount > m.amount)).unwrap() {
            stats.max = Some(MaxSwapInfo {
                amount: out_amount,
                initiator: initiator.to_owned(),
                time: env.block.time,
            })
        }
        Ok(stats)
    })?;

    // Update candlestick data
    OhlcBar::upsert(
        deps.storage,
        env.block.time,
        curve.calculate_quote_price()?,
        out_amount,
        in_amount,
    )?;

    let mut resp = Response::new().add_attributes(vec![
        attr("action", "buy"),
        attr("in_amount", in_amount_pre_fee.to_string()),
        attr("out_amount", out_amount.to_string()),
    ]);

    // Add submsg to send platform fee if exists
    if let Some(submg) = build_fee_transfer_submsg(deps.storage, &quote_token, fee_amount)? {
        resp = resp.add_submessage(submg);
    }

    // Add submsg to send purchased base tokens to initiator
    Ok(resp.add_submessage(base_token.transfer(&initiator, out_amount)?))
}

pub fn exec_sell(
    ctx: Context,
    msg: SellMsg,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    let SellMsg {
        initiator,
        min_out_amount,
    } = msg;

    let mut curve = CURVE.load(deps.storage)?;
    let quote_token = QUOTE_TOKEN.load(deps.storage)?;

    // Get amount to swap in
    let in_amount = amount;

    // Perform CP AMM swap
    let out_amount_pre_fee = curve.sell(in_amount, min_out_amount)?;

    // Compute sell-side platform fee
    let fee_amount = mul_pct_u128(out_amount_pre_fee, FEE_PCT_SELL.load(deps.storage)?)?;

    // Increment total historical aggregate fee amount
    NET_MAKER_FEE.update(deps.storage, |n| -> Result<_, ContractError> {
        add_u128(n, fee_amount)
    })?;

    // Subtract fee from amount recieved by sender
    let out_amount = sub_u128(out_amount_pre_fee, fee_amount)?;

    // Get initiator. The initiator is either the user performing the tx or user
    // on whose behalf the operator is performing it.
    let initiator = resolve_swap_initiator(deps.storage, deps.api, &info.sender, true, initiator, "sell")?;

    // Update initiator's account info
    ACCOUNT_STATS.update(deps.storage, &initiator, |maybe_stats| -> Result<_, ContractError> {
        let mut stats = maybe_stats.unwrap_or_default();
        stats.n_sells = add_u32(stats.n_sells, 1)?;
        stats.net_base_in = add_u128(stats.net_base_in, in_amount)?;
        stats.net_quote_out = add_u128(stats.net_quote_out, out_amount)?;
        Ok(stats)
    })?;

    // Update global stats
    MAKER_STATS.update(deps.storage, |mut stats| -> Result<_, ContractError> {
        stats.n = add_u64(stats.n, 1u64)?;
        if stats.max.is_none() || stats.max.clone().and_then(|m| Some(in_amount > m.amount)).unwrap() {
            stats.max = Some(MaxSwapInfo {
                amount: in_amount,
                initiator: initiator.to_owned(),
                time: env.block.time,
            })
        }
        Ok(stats)
    })?;

    // Update candlestick data
    OhlcBar::upsert(
        deps.storage,
        env.block.time,
        curve.calculate_quote_price()?,
        out_amount,
        in_amount,
    )?;

    let mut resp = Response::new().add_attributes(vec![
        attr("action", "sell"),
        attr("in_amount", in_amount.to_string()),
        attr("out_amount", out_amount.to_string()),
    ]);

    // Add submsg to send platform fee if exists
    if let Some(submg) = build_fee_transfer_submsg(deps.storage, &quote_token, fee_amount)? {
        resp = resp.add_submessage(submg);
    }

    // Add submsg to send purchased quote tokens to initiator
    Ok(resp.add_submessage(quote_token.transfer(&initiator, out_amount)?))
}

fn build_fee_transfer_submsg(
    store: &dyn Storage,
    quote_token: &Token,
    fee_amount: Uint128,
) -> Result<Option<SubMsg>, ContractError> {
    if !fee_amount.is_zero() {
        let fee_recipient = FEE_ADDR.load(store)?;
        return Ok(Some(quote_token.transfer(&fee_recipient, fee_amount)?));
    }
    Ok(None)
}
