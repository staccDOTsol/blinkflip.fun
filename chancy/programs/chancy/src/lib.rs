use anchor_lang::{prelude::*, solana_program::program_memory::sol_memcmp};
use std::{str::FromStr, vec};
use anchor_lang::solana_program::address_lookup_table::{
    instruction:: extend_lookup_table,
    state::AddressLookupTable,
};

declare_id!("76NigLJb5MPHMz4UyYHeAKR1v4Ck1SFrAkBjVKmbYJpA");

pub const BLOCK_HASHES: Pubkey = pubkey!("SysvarRecentB1ockHashes11111111111111111111");

#[program]
pub mod chancy {

    use anchor_lang::solana_program::sysvar::SysvarId;
    use anchor_lang::solana_program::{program::invoke, system_instruction};

    use super::*;
    pub fn commit(ctx: Context<Commit>, amount: u64) -> Result<()> {
        let user = &mut ctx.accounts.user_account;
        user.amount = amount;
        user.total_amount += amount;
        assert_eq!(user.state, GameState::Ready, "User state is not ready");
        user.last_play = Clock::get()?.unix_timestamp;
        user.state = GameState::Committed;
        user.streak += 1;
        user.user = *ctx.accounts.user.to_account_info().key;
        let referral_count = ctx.remaining_accounts.len() as u64;
        let streak_cap = 100 * (1_u64.saturating_sub(referral_count)) / 1_u64; // Adjusted to avoid division by zero

        // Ensure streak_cap is at least user.streak / 0.4
        let min_streak_cap = (user.streak as f64 / 0.4).ceil() as u64;
        let streak_cap = streak_cap.max(min_streak_cap);

        if user.streak > streak_cap {
            msg!("Sorry! You're streaking too high! Back to 0 for you.");
            user.streak = 0;
        }

        if user.streak > streak_cap  {
            msg!("Sorry! You're streaking too high! Bak to 0 for u..");
            user.streak = 0;
        }
        user.referral = ctx.accounts.referral.key();
        let house = &mut ctx.accounts.house;
        house.total_inflow += amount;
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
    pub fn reveal<'info>(ctx: Context<'_, '_, '_, 'info, Reveal<'info>>, ref_count: u8, lut_count: u8) -> Result<()> {
        let dev = &mut ctx.accounts.dev;
    let lookup_table_table = &mut ctx.accounts.lookup_table_table;
    let lookup = lookup_table_table.to_account_info();
    let remaining_accounts = ctx.remaining_accounts;
    let referrals = &remaining_accounts[remaining_accounts.len()-(ref_count+lut_count)as usize..remaining_accounts.len() - lut_count as usize];
    let luts = &remaining_accounts[remaining_accounts.len()-(lut_count)as usize..remaining_accounts.len() ];
    let mut pubkeys_to_add = vec![];
    let users = &remaining_accounts[0..remaining_accounts.len()-lut_count as usize-ref_count as usize];
    let named_accounts = vec![
        ctx.accounts.user_account.key(),
        ctx.accounts.house.key(),
        ctx.accounts.recent_blockhashes.key(),
        ctx.accounts.referral.key(),
    ];
    let mut lookup_table_combined_addresses = vec![];
    for account_info in luts.iter().rev() {
        let data = account_info.try_borrow_data()?;
        if let Ok(lookup_table) = AddressLookupTable::deserialize(&data) {
            if !lookup_table_table.lookup_tables.contains(&account_info.to_account_info().key) {
                let new_size = lookup.data.borrow().len() + 32;

                let rent = Rent::get()?;
                let new_minimum_balance = rent.minimum_balance(new_size);

                let lamports_diff = new_minimum_balance.saturating_sub(lookup.lamports());
                invoke(
                    &system_instruction::transfer(dev.to_account_info().key, lookup.key, lamports_diff),
                    &[
                        dev.to_account_info().clone(),
                        lookup.clone(),
                        ctx.accounts.system_program.to_account_info().clone(),
                    ],
                )?;

                lookup.realloc(new_size, false)?;

                lookup_table_table.lookup_tables.push(*account_info.to_account_info().key);
                
            }
            msg!("Found a valid Address Lookup Table: {:?}", lookup_table);
            lookup_table_combined_addresses.extend(lookup_table.addresses.iter());
        } else {
            msg!("Encountered an account that is not a valid Address Lookup Table, stopping iteration.");
            break;
        }
    }
    
        

        for named_account in named_accounts {
            if !lookup_table_combined_addresses.contains(&named_account) {
                pubkeys_to_add.push(named_account);
            }
        }
        if let Some(account_info) = luts.last() {
            let data = account_info.try_borrow_data()?;
            if let Ok(lookup_table) = AddressLookupTable::deserialize(&data) {
                msg!("Found final valid Address Lookup Table: {:?}", lookup_table);
                
                if lookup_table.addresses.len() < 255 - pubkeys_to_add.len() {
                    if !pubkeys_to_add.is_empty() {
                        let extend_instruction = extend_lookup_table(
                            *account_info.to_account_info().key,
                            *dev.to_account_info().key,
                            Some(*dev.to_account_info().key),
                            pubkeys_to_add,
                        );
                        invoke(
                            &extend_instruction,
                            &[
                                account_info.to_account_info(),
                                dev.to_account_info().clone(),
                            ],
                        )?;
                    }
                }
            } else {
                msg!("Encountered an account that is not a valid Address Lookup Table, stopping iteration.");
            }
        }
        
        let user = &mut ctx.accounts.user_account;
        let house = &mut ctx.accounts.house;
        // Verify the reveal is within the valid time window

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
            .checked_rem(1000)
            .ok_or(ErrorCode::InvalidModulus)? as f64;

        let rounded_amount = (user.amount as f64 / 10f64.powi(6)).round() as f64 / 2.0 / 1000.0;
        msg!("and now; magick: rounded, modded amounts: {}, {}", rounded_amount, modded);

        if modded <= rounded_amount  {

            let streak_cap = 100 * (1_u64.saturating_sub(ref_count as u64)) / 1_u64; // Adjusted to avoid division by zero

            // Ensure streak_cap is at least user.streak / 0.4
            let min_streak_cap = (user.streak as f64 / 0.4).ceil() as u64;
            let streak_cap = streak_cap.max(min_streak_cap);
    
            if user.streak > streak_cap {
                msg!("Sorry! You're streaking too high! Back to 0 for you.");
                user.streak = 0;
            }

            let snapshot = house.to_account_info().lamports();
            msg!("Winner winner chickum dinner");
            let base_winner_percentage = 50_f64;
            let streak_bonus_percentage = (user.streak as f64 / streak_cap as f64) * base_winner_percentage;
            let winner_reward = (snapshot.div_ceil(100) as f64 * (base_winner_percentage + streak_bonus_percentage)).round() as u64;



            house.recent_referrer = user.referral;
            house.recent_winner = *ctx.accounts.user.to_account_info().key;
            house.recent_won = winner_reward;
            house.recent_referrer_won = snapshot.div_ceil(10);
            house.total_won += winner_reward;
            house.total_wins += 1;
            house.recent_referral_chain = ref_count;

            transfer_service_fee_lamports(
                &house.to_account_info(),
                &ctx.accounts.user.to_account_info(),
                winner_reward
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
                for remaining_account in referrals.iter() {
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
                user.streak = 0;
        } else {
            msg!("No winner, no dinner");

            let  users_iter = users.iter();
            let  user_accounts_iter = users_iter.clone().skip(1);

            for (user, user_ai) in users_iter.zip(user_accounts_iter) {
    let user_account = User::try_from_slice(&user_ai.data.borrow())?;

// Assuming we're using a fixed-point representation with 9 decimal places (1 LAMPORT = 10^-9 SOL)
                const DECIMAL_PLACES: u64 = 9;
                const SCALE: u64 = 10u64.pow(DECIMAL_PLACES as u32);

                let user_info = user.to_account_info();
                let user_key = user_info.key;
                let house_lamports = house.total_inflow;
                let house_total_amount = house.to_account_info().lamports();

                if house_total_amount == 0 {
                    return Err(ProgramError::AccountBorrowFailed.into());
                }

                let weighted_lamports = house_lamports
                    .checked_mul(user_account.total_amount)
                    .and_then(|product| product.checked_mul(SCALE))
                    .and_then(|scaled_product| scaled_product.checked_div(house_total_amount))
                    .and_then(|result| result.checked_div(SCALE))
                    .ok_or(ProgramError::ArithmeticOverflow)?;

                msg!("Sending weighted lamports to user: {}", user_key);
                transfer_service_fee_lamports(
                    &house.to_account_info(),
                    &user_info,
                    weighted_lamports
                )?;
            }
        }
        user.state = GameState::Ready;
        Ok(())
  
}
}


#[derive(Accounts)]
pub struct Commit<'info> {
    #[account(
        mut,
        seeds = [b"house".as_ref(), dev.key().as_ref()],
        bump,
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
    #[account(init_if_needed, payer=user, seeds = [b"user", user.key().as_ref()], bump, space=8+138)]
    pub user_account: Account<'info, User>,
}

#[derive(Accounts)]
pub struct Reveal<'info> {
    #[account(
        mut,
        seeds = [b"house".as_ref(), dev.key().as_ref()],
        bump,
    )]
    pub house: Account<'info, House>,
    /// CHECK: This is the user account, not a signer
    #[account(mut)]
    pub user: AccountInfo<'info>,
    /// CHECK: This is safe as we only read from it
    pub recent_blockhashes: UncheckedAccount<'info>,
    #[account(mut,  address = Pubkey::from_str("GnTdsyRzW2pdAeWbfWwU2Uut6ghLfW6R1dsDUyDEUHUU").unwrap())]
    pub dev: Signer<'info>,
    #[account(mut)]
    /// CHECK:
    pub referral: AccountInfo<'info>,
    
    #[account(mut, seeds = [b"user", user.key().as_ref()], bump, constraint = user_account.state == GameState::Committed @ ErrorCode::InvalidState)]
    pub user_account: Account<'info, User>,
    #[account(init_if_needed, payer=dev, space=8+32+32+32, seeds = [b"lookup_table_table"], bump)]
    pub lookup_table_table: Account<'info, LookupTableTable>,
    pub system_program: Program<'info, System>

}

#[account]
pub struct LookupTableTable {
    pub lookup_tables: Vec<Pubkey>,
}

#[account]
pub struct House {
    pub recent_winner: Pubkey,
    pub recent_referrer: Pubkey,
    pub recent_won: u64,
    pub recent_referrer_won: u64,
    pub recent_referral_chain: u8,
    pub total_wins: u16,
    pub total_won: u64,
    pub total_inflow: u64
}



#[account]
pub struct User {
    pub referral: Pubkey,

    pub user: Pubkey,
    pub amount: u64,
    pub streak: u64,
    pub state: GameState,
    pub last_play: i64,
    pub total_amount: u64
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum GameState {
    #[default]
    Ready,
    Committed,
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