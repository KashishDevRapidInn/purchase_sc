use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
declare_id!("C6zXf83fM3aAac1t9AHh7AR6tYoMuB6nbYAhiMP7SV2K");

#[program]
pub mod purchase {
    use super::*;

    pub fn initialize_purchase(
        ctx: Context<InitializePurchase>,
        price: u64,
        name: String,
    ) -> Result<()> {
        let purchase_agreement = &mut ctx.accounts.purchase_agreement;
        purchase_agreement.seller = *ctx.accounts.user.key;
        purchase_agreement.buyer = Pubkey::default();
        purchase_agreement.price = price;
        purchase_agreement.status = AgreementStatus::ItemNotTransferred;
        purchase_agreement.item_name = name;
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
            &ctx.accounts.purchase_agreement.key(),
            amount,
        );

        invoke(
            &tx,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.purchase_agreement.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        let purchase_agreement = &mut ctx.accounts.purchase_agreement;
        purchase_agreement.status = AgreementStatus::PaymentDone;

        Ok(())
    }

    pub fn complete_purchase(ctx: Context<CompletePurchase>) -> Result<()> {
        let purchase_agreement = &ctx.accounts.purchase_agreement;

        if purchase_agreement.status == AgreementStatus::PurchaseCompleted {
            return Err(PurchaseErrors::PurchaseAlreadyCompleted.into());
        }

        if purchase_agreement.status == AgreementStatus::PaymentDone {
            let amount = purchase_agreement.price;
            let tx = system_instruction::transfer(
                &ctx.accounts.purchase_agreement.key(),
                &ctx.accounts.seller.key(),
                amount,
            );

            invoke(
                &tx,
                &[
                    ctx.accounts.purchase_agreement.to_account_info(),
                    ctx.accounts.seller.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        } else {
            return Err(PurchaseErrors::PaymentNotReceived.into());
        }
        let purchase_agreement = &mut ctx.accounts.purchase_agreement;
        purchase_agreement.status = AgreementStatus::PurchaseCompleted;

        Ok(())
    }
}

#[account]
pub struct PurchaseAgreement {
    pub price: u64,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub status: AgreementStatus,
    pub item_name: String,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AgreementStatus {
    ItemNotTransferred,
    PaymentDone,
    PurchaseCompleted,
}

#[derive(Accounts)]
pub struct InitializePurchase<'info> {
    #[account(init, payer = user, space = 8 + std::mem::size_of::<PurchaseAgreement>())]
    pub purchase_agreement: Account<'info, PurchaseAgreement>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MakePayment<'info> {
    #[account(mut)]
    pub purchase_agreement: Account<'info, PurchaseAgreement>,
    #[account(mut)]
    pub buyer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CompletePurchase<'info> {
    #[account(mut)]
    pub purchase_agreement: Account<'info, PurchaseAgreement>,
    #[account(mut)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum PurchaseErrors {
    BuyerDoNotHaveEnoughLamports,
    PurchaseAlreadyCompleted,
    PaymentNotReceived,
}
