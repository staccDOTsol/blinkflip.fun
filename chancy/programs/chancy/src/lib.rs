use anchor_lang::prelude::*;

declare_id!("76NigLJb5MPHMz4UyYHeAKR1v4Ck1SFrAkBjVKmbYJpA");
use anchor_lang::solana_program::program_memory::sol_memcmp;
use anchor_lang::solana_program::sysvar::{rent::Rent, slot_hashes, Sysvar};
use std::str::FromStr;
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
pub const BLOCK_HASHES: Pubkey =
anchor_lang::prelude::pubkey!("SysvarRecentB1ockHashes11111111111111111111");
#[program]
pub mod chancy {

    use anchor_lang::solana_program::{program::invoke, system_instruction};

    use super::*;

    pub fn flip(ctx: Context<Flip>, amount: u64) -> Result<()> {
        let recent_slothashes = &ctx.accounts.recent_blockhashes;

        if cmp_pubkeys(&recent_slothashes.key(), &BLOCK_HASHES) {
            msg!("recent_blockhashes is deprecated and will break soon");
        }
        if !cmp_pubkeys(&recent_slothashes.key(), &slot_hashes::id())
            && !cmp_pubkeys(&recent_slothashes.key(), &BLOCK_HASHES)
        {
            return Err(ProgramError::InvalidInstructionData.into());
        }


        let data = recent_slothashes.data.borrow();
        let clock = Clock::get()?;
        
        let timestamp = clock.unix_timestamp;

        let modded: usize = timestamp
        .checked_rem(16)
        .ok_or(ProgramError::InvalidInstructionData)? as usize;


        let most_recent = arrayref::array_ref![data, modded, 8];

        let index = u64::from_le_bytes(*most_recent);
        let modded: f64 = index
                .checked_rem(100)
                .ok_or(ProgramError::InvalidInstructionData)? as f64;

        let rounded_amount = (amount as f64 / 10f64.powi(9)).round() as f64 / 2.0;
        msg!("and now; magick: rounded, modded amounts: {}, {}", rounded_amount, modded);

        if modded <= rounded_amount  {
            msg!("Winner winner chickum dinner");
            transfer_service_fee_lamports(
                &ctx.accounts.house.to_account_info(),
                &ctx.accounts.signer.to_account_info(),
                ctx.accounts.house.get_lamports().div_ceil(2)
            )?;
            transfer_service_fee_lamports(
                &ctx.accounts.house.to_account_info(),
                &ctx.accounts.dev.to_account_info(),
                ctx.accounts.house.get_lamports().div_ceil(8)
            )?;
            transfer_service_fee_lamports(
                &ctx.accounts.house.to_account_info(),
                &ctx.accounts.referral.to_account_info(),
                ctx.accounts.house.get_lamports().div_ceil(8)
            )?;
        }
        else {
            msg!("No winner, no dinner");
            let ix = system_instruction::transfer(ctx.accounts.signer.to_account_info().key, ctx.accounts.house.to_account_info().key, amount);
            invoke(&ix, &[ctx.accounts.signer.to_account_info(), ctx.accounts.house.to_account_info()])?;
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Flip<'info> {
    /// CHECK:
    pub recent_blockhashes: UncheckedAccount<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init_if_needed, space=8+320, payer=signer, seeds=[b"house", dev.key().as_ref()], bump)]
    pub house: Account<'info, House>,
    pub system_program: Program<'info, System>,
    /// CHECK:
    #[account(signer, address=Pubkey::from_str("GnTdsyRzW2pdAeWbfWwU2Uut6ghLfW6R1dsDUyDEUHUU").unwrap())]
    pub dev: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub referral: AccountInfo<'info>
}
#[account]
pub struct House {

}