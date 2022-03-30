use anchor_spl::token::Token;
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, TokenAccount, Mint},
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod nft_lending {
    use super::*;

    pub fn loan(
        ctx: Context<Loan>,
        bump: u8,
        loan_amount: u64,
        collateral_amount: u64,
        default_at: i64,
        borrower: Option<Pubkey>,
    ) -> ProgramResult {
        if loan_amount == 0 {
            return Err(NftLendingError::LoanCannotBeZero.into());
        }
        if collateral_amount == 0 {
            return Err(NftLendingError::CollateralCannotBeZero.into());
        }

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.lender_token_account.to_account_info(),
                    to: ctx.accounts.loan.to_account_info(),
                    authority: ctx.accounts.lender.to_account_info(),
                },
            ),
            loan_amount,
        )?;

        let loan_agreement = &mut ctx.accounts.loan_agreement;
        loan_agreement.bump = bump;
        loan_agreement.loan_amount = loan_amount;
        loan_agreement.default_at = default_at;
        loan_agreement.collateral_amount = collateral_amount;
        loan_agreement.borrower = borrower;
        loan_agreement.borrowed = false;
        Ok(())
    }

    pub fn borrow(
        ctx: Context<Borrow>,
        expected_amount: u64,
        collateral_amount: u64,
    ) -> ProgramResult {
        let loan_agreement = &ctx.accounts.loan_agreement;
        if loan_agreement.collateral_amount != collateral_amount || ctx.accounts.loan.amount != expected_amount {
            return Err(NftLendingError::UnexpectedLoanAgreement.into());
        }

        match loan_agreement.borrower {
            Some(borrower) => {
                if borrower != ctx.accounts.borrower.key() {
                    return Err(NftLendingError::IncorrectBorrower.into());
                }
            },
            _ => (),
        }

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.borrower_collateral_token_account.to_account_info(),
                    to: ctx.accounts.collateral.to_account_info(),
                    authority: ctx.accounts.borrower.to_account_info(),
                },
            ),
            loan_agreement.collateral_amount,
        )?;

        let loan_agreement_pk = loan_agreement.key();
        let seeds = &[loan_agreement_pk.as_ref(), b"authority".as_ref(), &[loan_agreement.bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info().clone(),
                token::Transfer {
                    from: ctx.accounts.loan.to_account_info(),
                    to: ctx.accounts.borrower_loan_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                &[&seeds[..]]
            ),
            ctx.accounts.loan_agreement.loan_amount,
        )?;

        ctx.accounts.loan_agreement.borrowed = true;

        Ok(())
    }

    pub fn repay(
        ctx: Context<Repay>,
    ) -> ProgramResult {
        let loan_agreement = &ctx.accounts.loan_agreement;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.borrower_loan_token_account.to_account_info(),
                    to: ctx.accounts.loan.to_account_info(),
                    authority: ctx.accounts.borrower.to_account_info(),
                },
            ),
            loan_agreement.loan_amount,
        )?;

        let loan_agreement_pk = loan_agreement.key();
        let seeds = &[loan_agreement_pk.as_ref(), b"authority".as_ref(), &[loan_agreement.bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info().clone(),
                token::Transfer {
                    from: ctx.accounts.collateral.to_account_info(),
                    to: ctx.accounts.borrower_collateral_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                &[&seeds[..]]
            ),
            ctx.accounts.loan_agreement.collateral_amount,
        )?;

        ctx.accounts.loan_agreement.borrowed = false;

        Ok(())
    }

    pub fn close(
        ctx: Context<Close>,
    ) -> ProgramResult {
        let loan_agreement = &ctx.accounts.loan_agreement;

        if loan_agreement.borrowed && Clock::get()?.unix_timestamp < loan_agreement.default_at {
            return Err(NftLendingError::DefaultAtIsNotReached.into());
        }

        let loan_agreement_pk = loan_agreement.key();
        let seeds = &[loan_agreement_pk.as_ref(), b"authority".as_ref(), &[loan_agreement.bump]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info().clone(),
                token::Transfer {
                    from: ctx.accounts.collateral.to_account_info(),
                    to: ctx.accounts.lender_collateral_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                &[&seeds[..]]
            ),
            ctx.accounts.collateral.amount, // Empty collateral
        )?;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info().clone(),
                token::Transfer {
                    from: ctx.accounts.loan.to_account_info(),
                    to: ctx.accounts.lender_loan_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                &[&seeds[..]]
            ),
            ctx.accounts.loan.amount, // Empty loan
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Loan<'info> {
    #[account(
        init,
        payer = lender
    )]
    loan_agreement: Account<'info, LoanAgreement>,
    #[account(
        seeds = [loan_agreement.key().as_ref(), b"authority"],
        bump,
    )]
    authority: UncheckedAccount<'info>,
    #[account(
        init,
        seeds = [loan_agreement.key().as_ref(), b"loan"],
        bump,
        token::mint = loan_mint,
        token::authority = authority,
        payer = lender
    )]
    loan: Account<'info, TokenAccount>,
    loan_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        seeds = [loan_agreement.key().as_ref(), b"collateral"],
        bump,
        token::mint = collateral_mint,
        token::authority = authority,
        payer = lender,
    )]
    collateral: Account<'info, TokenAccount>,
    collateral_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    lender: Signer<'info>,
    #[account(mut)]
    lender_token_account: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    loan_agreement: Account<'info, LoanAgreement>,
    #[account(
        seeds = [loan_agreement.key().as_ref(), b"authority"],
        bump,
    )]
    authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [loan_agreement.key().as_ref(), b"collateral"],
        bump,
    )]
    collateral: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [loan_agreement.key().as_ref(), b"loan"],
        bump,
    )]
    loan: Account<'info, TokenAccount>,

    borrower: Signer<'info>,
    #[account(mut)]
    borrower_loan_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    borrower_collateral_token_account: Account<'info, TokenAccount>,

    token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(mut)]
    loan_agreement: Account<'info, LoanAgreement>,
    #[account(
        seeds = [loan_agreement.key().as_ref(), b"authority"],
        bump,
    )]
    authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [loan_agreement.key().as_ref(), b"collateral"],
        bump,
    )]
    collateral: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [loan_agreement.key().as_ref(), b"loan"],
        bump,
    )]
    loan: Account<'info, TokenAccount>,

    borrower: Signer<'info>,
    #[account(mut)]
    borrower_loan_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    borrower_collateral_token_account: Account<'info, TokenAccount>,

    token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut, close = lender, has_one = lender)]
    loan_agreement: Account<'info, LoanAgreement>,
    #[account(
        seeds = [loan_agreement.key().as_ref(), b"authority"],
        bump,
    )]
    authority: UncheckedAccount<'info>,
    #[account(
        mut,
        close = lender,
        seeds = [loan_agreement.key().as_ref(), b"collateral"],
        bump,
    )]
    collateral: Account<'info, TokenAccount>,
    #[account(
        mut,
        close = lender,
        seeds = [loan_agreement.key().as_ref(), b"loan"],
        bump,
    )]
    loan: Account<'info, TokenAccount>,

    #[account(mut)]
    lender: Signer<'info>,
    #[account(mut)]
    lender_loan_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    lender_collateral_token_account: Account<'info, TokenAccount>,

    token_program: Program<'info, Token>,
}

#[account]
#[derive(Default)]
pub struct LoanAgreement {
    bump: u8,
    lender: Pubkey,
    borrower: Option<Pubkey>, // Borrower can be constrained or not by the loan agreement
    loan_amount: u64,
    collateral_amount: u64,
    default_at: i64,
    borrowed: bool,
}

#[error]
pub enum NftLendingError {
    #[msg("Loan cannot be zero")]
    LoanCannotBeZero,
    #[msg("Collateral cannot be zero")]
    CollateralCannotBeZero,
    #[msg("Unexpected loan agreement")]
    UnexpectedLoanAgreement,
    #[msg("Default at is not reached")]
    DefaultAtIsNotReached,
    #[msg("Incorrect borrower")]
    IncorrectBorrower,
}
