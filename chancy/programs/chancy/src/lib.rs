use anchor_lang::{prelude::*, solana_program::program_memory::sol_memcmp};
use std::str::FromStr;
declare_id!("76NigLJb5MPHMz4UyYHeAKR1v4Ck1SFrAkBjVKmbYJpA");

pub const BLOCK_HASHES: Pubkey = pubkey!("SysvarRecentB1ockHashes11111111111111111111");

#[program]
pub mod chancy {

    use anchor_lang::solana_program::sysvar::SysvarId;

    use super::*;
    pub fn commit(ctx: Context<Commit>, amount: u64) -> Result<()> {
        let house = &mut ctx.accounts.house;
        house.user = ctx.accounts.user.key();
        house.amount = amount;
        house.commit_slot = Clock::get()?.slot;
        house.state = GameState::Committed;
        let user_account = &mut ctx.accounts.user_account;
        user_account.referral = ctx.accounts.referral.key();

        // Transfer the bet amount from user to house account
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: house.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, amount)?;

        Ok(())
    }
    pub fn reveal(ctx: Context<Reveal>) -> Result<()> {
        let house = &mut ctx.accounts.house;

        // Verify the reveal is within the valid time window
        let current_slot = Clock::get()?.slot;
        require!(current_slot - house.commit_slot <= 100, ErrorCode::RevealTooLate);


        let recent_slothashes = &ctx.accounts.recent_blockhashes;

        if cmp_pubkeys(&recent_slothashes.key(), &BLOCK_HASHES) {
            msg!("recent_blockhashes is deprecated and will break soon");
        }
        if !cmp_pubkeys(&recent_slothashes.key(), &SlotHashes::id())
            && !cmp_pubkeys(&recent_slothashes.key(), &BLOCK_HASHES)
        {
            return Err(ErrorCode::InvalidBlockhashes.into());
        }

        let data = recent_slothashes.data.borrow();
        let clock = Clock::get()?;
        
        let timestamp = clock.unix_timestamp;

        let modded: usize = timestamp
            .checked_rem(16)
            .ok_or(ErrorCode::InvalidModulus)? as usize;

        let most_recent = arrayref::array_ref![data, modded, 8];

        let index = u64::from_le_bytes(*most_recent);
        let modded: f64 = index
            .checked_rem(100)
            .ok_or(ErrorCode::InvalidModulus)? as f64;

        let rounded_amount = (house.amount as f64 / 10f64.powi(9)).round() as f64 / 2.0;
        msg!("and now; magick: rounded, modded amounts: {}, {}", rounded_amount, modded);

        if modded <= rounded_amount  {
            let snapshot = house.to_account_info().lamports();
            msg!("Winner winner chickum dinner");
            transfer_service_fee_lamports(
                &house.to_account_info(),
                &ctx.accounts.user.to_account_info(),
                snapshot.div_ceil(2) 
            )?;
            transfer_service_fee_lamports(
                &house.to_account_info(),
                &ctx.accounts.dev.to_account_info(),
                snapshot.div_ceil(10)
            )?;
            transfer_service_fee_lamports(
                &house.to_account_info(),
                &ctx.accounts.referral.to_account_info(),
                snapshot.div_ceil(10) 
                )?;
                let mut prev_referral = ctx.accounts.referral.to_account_info().key;
                for remaining_account in ctx.remaining_accounts.iter() {
                    let remaining_user = User::try_from_slice(&remaining_account.data.borrow())?;
                    let referral = remaining_user.referral;
                    msg!("Referral: {}", referral);

                    let referral_account = remaining_account.to_account_info();
                    if !cmp_pubkeys(&prev_referral, &referral_account.key) {
                        transfer_service_fee_lamports(
                            &house.to_account_info(),
                            &referral_account,
                            house.to_account_info().lamports().div_ceil(20)
                        )?;
                    }
                    prev_referral = referral_account.key;
                }
        } else {
            msg!("No winner, no dinner");
            // Funds already in house account, no transfer needed
        }

        house.state = GameState::Revealed;
        Ok(())
    }
}



#[derive(Accounts)]
pub struct Commit<'info> {
    #[account(
        mut,
        seeds = [b"house".as_ref(), dev.key().as_ref()],
        bump,
        constraint = house.state == GameState::Ready @ ErrorCode::HouseNotReady
    )]
    pub house: Account<'info, House>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(mut,  address = Pubkey::from_str("GnTdsyRzW2pdAeWbfWwU2Uut6ghLfW6R1dsDUyDEUHUU").unwrap())]
    /// CHECK:
    pub dev: Signer<'info>,
    /// CHECK:
    pub referral: AccountInfo<'info>,
    #[account(init_if_needed, payer=user, seeds = [b"user".as_ref(), user.key().as_ref()], bump, space=8+32)]
    pub user_account: Account<'info, User>,
}

#[derive(Accounts)]
pub struct Reveal<'info> {
    #[account(
        mut,
        seeds = [b"house".as_ref(), dev.key().as_ref()],
        bump,
        constraint = house.state == GameState::Committed @ ErrorCode::InvalidState
    )]
    pub house: Account<'info, House>,
    /// CHECK: This is the user account, not a signer
    #[account(mut, address = house.user @ ErrorCode::InvalidUser)]
    pub user: AccountInfo<'info>,
    /// CHECK: This is safe as we only read from it
    pub recent_blockhashes: UncheckedAccount<'info>,
    #[account(mut,  address = Pubkey::from_str("GnTdsyRzW2pdAeWbfWwU2Uut6ghLfW6R1dsDUyDEUHUU").unwrap())]
    pub dev: Signer<'info>,
    #[account(mut)]
    /// CHECK:
    pub referral: AccountInfo<'info>,
    
    #[account(mut, seeds = [b"user".as_ref(), user.key().as_ref()], bump)]
    pub user_account: Account<'info, User>,
}


#[account]
pub struct House {
    pub user: Pubkey,
    pub amount: u64,
    pub commit_slot: u64,
    pub state: GameState,
}



#[account]
pub struct User {
    pub referral: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Ready,
    Committed,
    Revealed,
}

#[error_code]
pub enum ErrorCode {
    RevealTooLate,
    InvalidState,
    HouseNotReady,
    InvalidUser,
    InvalidModulus,
    InvalidBlockhashes
}

fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), 32) == 0
}

fn transfer_service_fee_lamports(
    from_account: &AccountInfo,
    to_account: &AccountInfo,
    amount_of_lamports: u64,
) -> Result<()> {
    // Does the from account have enough lamports to transfer?
    if **from_account.try_borrow_lamports()? < amount_of_lamports {
        return Err(ProgramError::InsufficientFunds.into());
    }
    // Debit from_account and credit to_account
    **from_account.try_borrow_mut_lamports()? -= amount_of_lamports;
    **to_account.try_borrow_mut_lamports()? += amount_of_lamports;
    Ok(())
}