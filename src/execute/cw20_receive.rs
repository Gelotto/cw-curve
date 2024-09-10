use crate::{
    error::ContractError,
    msg::Cw20ReceiveInnerMsg,
    state::{BASE_TOKEN, OPERATOR_ADDR, QUOTE_TOKEN},
    token::Token,
};
use cosmwasm_std::{ensure_eq, from_json, Addr, Response};
use cw20::Cw20ReceiveMsg;

use super::{
    buy_sell::{exec_buy, exec_sell},
    Context,
};

pub fn exec_cw20_receive(
    ctx: Context,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let Cw20ReceiveMsg {
        sender: token_sender,
        amount,
        msg,
    } = msg;

    let token_sender = ctx.deps.api.addr_validate(token_sender.as_str())?;
    let mut using_operator = false;

    if let Some(operator_addr) = OPERATOR_ADDR.may_load(ctx.deps.storage)? {
        using_operator = true;
        ensure_eq!(
            operator_addr,
            token_sender,
            ContractError::NotAuthorized {
                reason: "Only the defined operator can buy and sell".to_owned()
            }
        )
    }

    let quote_token = QUOTE_TOKEN.load(ctx.deps.storage)?;

    match from_json::<Cw20ReceiveInnerMsg>(msg.as_slice())? {
        Cw20ReceiveInnerMsg::Buy(mut msg) => {
            ensure_is_authorized_cw20(&quote_token, &ctx.info.sender)?;
            if !using_operator {
                msg.initiator = Some(token_sender);
            }
            exec_buy(ctx, msg, Some(amount))
        },
        Cw20ReceiveInnerMsg::Sell(mut msg) => {
            let base_token = BASE_TOKEN.load(ctx.deps.storage)?;
            ensure_is_authorized_cw20(&base_token, &ctx.info.sender)?;
            if !using_operator {
                msg.initiator = Some(token_sender);
            }
            exec_sell(ctx, msg, amount)
        },
    }
}

fn ensure_is_authorized_cw20(
    exp_token: &Token,
    cw20_addr: &Addr,
) -> Result<(), ContractError> {
    if let Some(exp_cw20_addr) = exp_token.get_address() {
        ensure_eq!(
            exp_cw20_addr,
            cw20_addr,
            ContractError::NotAuthorized {
                reason: "Received unrecognized cw20 token".to_string()
            }
        )
    } else {
        return Err(ContractError::NotAuthorized {
            reason: "Curve token is not a cw20".to_string(),
        });
    }
    Ok(())
}
