use cosmwasm_std::{attr, Coin, Response, Storage, SubMsg, Uint128};

use crate::{
    error::ContractError,
    math::{add_u32, mul_pct_u128, sub_u128},
    models::{account::AccountStats, ohlc::OhlcBar},
    msg::SwapMsg,
    state::{ACCOUNT_STATS, BASE_TOKEN, BUY_FEE_PCT, CURVE, FEE_RECIPIENT_ADDR, QUOTE_TOKEN, SELL_FEE_PCT},
    token::Token,
    utils::resolve_swap_initiator,
};

use super::Context;

pub fn exec_buy(
    ctx: Context,
    msg: SwapMsg,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    let SwapMsg {
        amount,
        initiator,
        min_out_amount,
        token,
    } = msg;

    let mut curve = CURVE.load(deps.storage)?;
    let quote_token = QUOTE_TOKEN.load(deps.storage)?;

    // Get amount we're trying to swap in
    let in_amount_pre_fee = if token == quote_token {
        amount
    } else {
        curve.to_quote_amount(amount)?
    };

    // Ensure payment
    ensure_payment_in_funds(&info.funds, &quote_token, &token, in_amount_pre_fee)?;

    // Compute buy or sell-side platform fee
    let fee_amount = mul_pct_u128(in_amount_pre_fee, BUY_FEE_PCT.load(deps.storage)?)?;

    // Subtract fee from amount recieved by sender
    let in_amount = sub_u128(in_amount_pre_fee, fee_amount)?;

    // Perform AMM swap
    let out_amount = curve.buy(in_amount, min_out_amount)?;

    // Get initiator. The initiator is either the user performing the tx or the
    // user on whose behalf the operator is performing it.
    let initiator = resolve_swap_initiator(deps.storage, deps.api, &info.sender, initiator, "buy")?;

    // Update initiator's account info
    ACCOUNT_STATS.update(deps.storage, &initiator, |maybe_stats| -> Result<_, ContractError> {
        let mut stats = maybe_stats.unwrap_or_else(|| AccountStats { n_buys: 0, n_sells: 0 });
        stats.n_buys = add_u32(stats.n_buys, 1)?;
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

    Ok(resp)
}

pub fn exec_sell(
    ctx: Context,
    msg: SwapMsg,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    let SwapMsg {
        amount,
        initiator,
        min_out_amount,
        token,
    } = msg;

    let mut curve = CURVE.load(deps.storage)?;
    let quote_token = QUOTE_TOKEN.load(deps.storage)?;
    let base_token = BASE_TOKEN.load(deps.storage)?;

    // Get amount to swap in
    let in_amount = if token == quote_token {
        curve.to_base_amount(amount)?
    } else {
        amount
    };

    // Ensure payment
    ensure_payment_in_funds(&info.funds, &base_token, &token, in_amount)?;

    // Perform CP AMM swap
    let out_amount_pre_fee = curve.sell(in_amount, min_out_amount)?;

    // Compute sell-side platform fee
    let fee_amount = mul_pct_u128(out_amount_pre_fee, SELL_FEE_PCT.load(deps.storage)?)?;

    // Subtract fee from amount recieved by sender
    let out_amount = sub_u128(out_amount_pre_fee, fee_amount)?;

    // Get initiator. The initiator is either the user performing the tx or user
    // on whose behalf the operator is performing it.
    let initiator = resolve_swap_initiator(deps.storage, deps.api, &info.sender, initiator, "sell")?;

    // Update initiator's account info
    ACCOUNT_STATS.update(deps.storage, &initiator, |maybe_stats| -> Result<_, ContractError> {
        let mut stats = maybe_stats.unwrap_or_else(|| AccountStats { n_buys: 0, n_sells: 0 });
        stats.n_sells = add_u32(stats.n_sells, 1)?;
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

    Ok(resp)
}

fn build_fee_transfer_submsg(
    store: &dyn Storage,
    quote_token: &Token,
    fee_amount: Uint128,
) -> Result<Option<SubMsg>, ContractError> {
    if !fee_amount.is_zero() {
        let fee_recipient = FEE_RECIPIENT_ADDR.load(store)?;
        return Ok(Some(quote_token.transfer(&fee_recipient, fee_amount)?));
    }
    Ok(None)
}

fn ensure_payment_in_funds(
    funds: &Vec<Coin>,
    exp_token: &Token,
    in_token: &Token,
    exp_amount: Uint128,
) -> Result<(), ContractError> {
    if let Token::Denom(denom) = in_token {
        if let Some(coin) = exp_token.find_in_funds(&funds, None) {
            if coin.amount != exp_amount {
                return Err(ContractError::InsufficientFunds {
                    exp_amount: exp_amount.into(),
                    amount: coin.amount.into(),
                    denom: denom.to_owned(),
                });
            }
        } else {
            return Err(ContractError::InsufficientFunds {
                exp_amount: exp_amount.into(),
                amount: 0,
                denom: denom.to_owned(),
            });
        }
    } else {
        // NOTE: If the base token is a CW20, then we assume that payment has
        // been validated by the caller, specifically in the Cw20Receiver
        // interface. Hence, we do nothing here in this case.
    }

    Ok(())
}
