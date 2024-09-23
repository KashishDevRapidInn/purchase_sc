use anchor_lang::prelude::*;

use anchor_spl::token::Mint;
use anchor_spl::token::{self, Token, TokenAccount};
declare_id!("C6zXf83fM3aAac1t9AHh7AR6tYoMuB6nbYAhiMP7SV2K");

#[program]
pub mod purchase {
    use super::*;
    use anchor_lang::solana_program::program::invoke;
    use anchor_lang::solana_program::system_instruction;
    use anchor_spl::token::{self, Approve};

    pub fn initialize_purchase(
        ctx: Context<InitializePurchase>,
        price: u64,
        nft_id: Pubkey,
        start_time: u64,
        end_time: u64,
    ) -> Result<()> {
        let purchase_agreement = &mut ctx.accounts.purchase_agreement;
        // let (agreement_pda, bump) = Pubkey::find_program_address(
        //     &[b"purchase_agreement", nft_id.as_bytes()],
        //     ctx.program_id,
        // );

        purchase_agreement.seller = *ctx.accounts.seller.key;
        purchase_agreement.buyer = None;
        purchase_agreement.price = price;
        purchase_agreement.status = AgreementStatus::ItemNotTransferred;
        purchase_agreement.nft_id = nft_id;
        purchase_agreement.start_time = start_time;
        purchase_agreement.end_time = end_time;
        purchase_agreement.nft_status = NftStatus::Active;
        // purchase_agreement.pda = agreement_pda;

        let cpi_accounts = Approve {
            to: ctx.accounts.nft_holding_account.to_account_info(),
            delegate: ctx.accounts.purchase_agreement.to_account_info(), //state a/c
            authority: ctx.accounts.seller.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::approve(cpi_ctx, 1)?; // Adjust the amount if necessary
        Ok(())
    }
    pub fn make_payment(ctx: Context<MakePayment>) -> Result<()> {
        let purchase_agreement = &ctx.accounts.purchase_agreement;
        let amount = purchase_agreement.price;

        let buyer_account_info = &ctx.accounts.buyer.to_account_info();
        let buyer_lamports = buyer_account_info.lamports();

        if buyer_lamports < amount {
            return Err(PurchaseErrors::BuyerDoNotHaveEnoughLamports.into());
        }

        let tx = system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &ctx.accounts.seller.key(),
            amount,
        );

        invoke(
            &tx,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.seller.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        let purchase_agreement = &mut ctx.accounts.purchase_agreement;
        purchase_agreement.buyer = Some(*ctx.accounts.buyer.key);
        purchase_agreement.status = AgreementStatus::PaymentDone;
        purchase_agreement.nft_status = NftStatus::Sold;

        let cpi_accounts = token::Transfer {
            from: ctx.accounts.nft_holding_account.to_account_info(),
            to: ctx.accounts.buyer_nft_account.to_account_info(),
            authority: purchase_agreement.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, 1)?;
        purchase_agreement.status = AgreementStatus::PurchaseCompleted;
        Ok(())
    }
}

#[account]
pub struct PurchaseAgreement {
    pub price: u64,
    pub seller: Pubkey,
    pub buyer: Option<Pubkey>,
    pub status: AgreementStatus,
    pub nft_id: Pubkey,
    pub start_time: u64,
    pub end_time: u64,
    pub nft_status: NftStatus,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AgreementStatus {
    ItemNotTransferred,
    PaymentDone,
    PurchaseCompleted,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum NftStatus {
    Active,
    Sold,
    NotAvailable,
}

#[derive(Accounts)]
pub struct InitializePurchase<'info> {
    #[account(init, payer = seller, space = 144)]
    pub purchase_agreement: Account<'info, PurchaseAgreement>,
    #[account(mut)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub nft_account: Account<'info, token::TokenAccount>,
    #[account(address = nft_account.mint)] // Ensure this matches the mint of the NFT
    pub nft_mint: Account<'info, Mint>, // Mint account for the NFT
    pub token_program: Program<'info, Token>,
    #[account(init, payer = seller, token::mint = nft_mint, token::authority = seller)]
    pub nft_holding_account: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct MakePayment<'info> {
    #[account(mut)]
    pub purchase_agreement: Account<'info, PurchaseAgreement>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub buyer_nft_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub nft_holding_account: Account<'info, TokenAccount>,
}

#[error_code]
pub enum PurchaseErrors {
    BuyerDoNotHaveEnoughLamports,
    PurchaseAlreadyCompleted,
    PaymentNotReceived,
}

// active, sold, Not available
