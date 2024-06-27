use anchor_lang::{prelude::*, solana_program::program_memory::sol_memcmp};
use std::{str::FromStr, vec};
use anchor_lang::solana_program::address_lookup_table::{
    instruction::extend_lookup_table,
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
        user.update_commit(amount, &ctx.accounts.user, &ctx.accounts.referral)?;

        let house = &mut ctx.accounts.house;
        house.update_inflow(amount);

        transfer_lamports(
            &ctx.accounts.user,
            &house.to_account_info(),
            amount,
            &ctx.accounts.system_program.to_account_info().key,
        )?;

        Ok(())
    }

    pub fn reveal<'info>(
        ctx: Context<'_, '_, '_, 'info, Reveal<'info>>,
        ref_count: u8,
        lut_count: u8,
    ) -> Result<()> {
        let dev = &mut ctx.accounts.dev;
        let lookup_table_table = &mut ctx.accounts.lookup_table_table;
        let lookup = lookup_table_table.to_account_info();

        let (referrals, luts, users) = parse_remaining_accounts(&ctx.remaining_accounts, ref_count, lut_count);

        update_lookup_tables(luts, lookup_table_table, lookup, dev, &ctx.accounts.system_program)?;

        let named_accounts = vec![
            ctx.accounts.user_account.key(),
            ctx.accounts.house.key(),
            ctx.accounts.recent_blockhashes.key(),
            ctx.accounts.referral.key(),
        ];

        add_missing_accounts_to_lookup_table(luts, named_accounts, dev)?;

        let user = &mut ctx.accounts.user_account;
        let house = &mut ctx.accounts.house;

        let recent_slothashes = &ctx.accounts.recent_blockhashes;
        validate_blockhashes(recent_slothashes)?;

        let (modded, rounded_amount) = calculate_modded_and_rounded_amount(recent_slothashes, user)?;

        if modded <= rounded_amount {
            process_win(
                user,
                house,
                &ctx.accounts.user,
                &ctx.accounts.dev,
                &ctx.accounts.referral,
                referrals,
                ref_count,
            )?;
        } else {
            process_loss(users, house)?;
        }

        user.state = GameState::Ready;
        Ok(())
    }
}

// ... remaining code ...

fn parse_remaining_accounts<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    ref_count: u8,
    lut_count: u8,
) -> (&[AccountInfo<'info>], &[AccountInfo<'info>], &[AccountInfo<'info>]) {
    let total_count = remaining_accounts.len();
    let referrals_end = total_count - lut_count as usize;
    let luts_start = referrals_end;
    let users_end = total_count - lut_count as usize - ref_count as usize;

    let referrals = &remaining_accounts[users_end..referrals_end];
    let luts = &remaining_accounts[luts_start..total_count];
    let users = &remaining_accounts[0..users_end];

    (referrals, luts, users)
}

fn update_lookup_tables<'info>(
    luts: &[AccountInfo<'info>],
    lookup_table_table: &mut Account<'info, LookupTableTable>,
    lookup: AccountInfo<'info>,
    dev: &Signer<'info>,
    system_program: &Program<'info, System>,
) -> Result<()> {
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
                        system_program.to_account_info().clone(),
                    ],
                )?;

                lookup.realloc(new_size, false)?;
                lookup_table_table.lookup_tables.push(*account_info.to_account_info().key);
            }
        } else {
            break;
        }
}
Ok(())
}

fn add_missing_accounts_to_lookup_table<'info>(
luts: &[AccountInfo<'info>],
named_accounts: Vec<Pubkey>,
dev: &Signer<'info>,
) -> Result<()> {
if let Some(account_info) = luts.last() {
    let data = account_info.try_borrow_data()?;
    if let Ok(lookup_table) = AddressLookupTable::deserialize(&data) {
        let mut pubkeys_to_add = vec![];
        for named_account in named_accounts {
            if !lookup_table.addresses.contains(&named_account) {
                pubkeys_to_add.push(named_account);
            }
        }

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
    }
}
Ok(())
}

fn validate_blockhashes(recent_slothashes: &UncheckedAccount) -> Result<()> {
if cmp_pubkeys(&recent_slothashes.key(), &BLOCK_HASHES) {
    msg!("recent_blockhashes is deprecated and will break soon");
}
if !cmp_pubkeys(&recent_slothashes.key(), &SlotHashes::id())
    && !cmp_pubkeys(&recent_slothashes.key(), &BLOCK_HASHES)
{
    return Err(ErrorCode::InvalidBlockhashes.into());
}
Ok(())
}

fn calculate_modded_and_rounded_amount(
recent_slothashes: &UncheckedAccount,
user: &User,
) -> Result<(f64, f64)> {
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
Ok((modded, rounded_amount))
}

fn process_win<'info>(
user: &mut User,
house: &mut House,
user_account: &AccountInfo<'info>,
dev: &AccountInfo<'info>,
referral: &AccountInfo<'info>,
referrals: &[AccountInfo<'info>],
ref_count: u8,
) -> Result<()> {
let streak_cap = calculate_streak_cap(ref_count, user.streak);
if user.streak > streak_cap {
    msg!("Sorry! You're streaking too high! Back to 0 for you.");
    user.streak = 0;
}

let snapshot = house.to_account_info().lamports();
let winner_reward = calculate_winner_reward(snapshot, user.streak, streak_cap);

house.update_win(
    *user_account.to_account_info().key,
    winner_reward,
    snapshot.div_ceil(10),
    *referral.key,
    ref_count,
);

transfer_lamports(
    &house.to_account_info(),
    user_account,
    winner_reward,
    &anchor_lang::solana_program::system_program::id(),
)?;
transfer_lamports(
    &house.to_account_info(),
    dev,
    snapshot.div_ceil(10),
    &anchor_lang::solana_program::system_program::id(),
)?;
transfer_lamports(
    &house.to_account_info(),
    referral,
    snapshot.div_ceil(10),
    &anchor_lang::solana_program::system_program::id(),
)?;

process_referral_rewards(referrals, house)?;

user.streak = 0;
Ok(())
}

fn process_loss<'info>(
users: &[AccountInfo<'info>],
house: &mut House,
) -> Result<()> {
for (user, user_ai) in users.iter().zip(users.iter().skip(1)) {
    let user_info = user.to_account_info();
    let user_account = User::try_from_slice(&user_ai.data.borrow())?;

    let weighted_lamports = calculate_weighted_lamports(
        house.total_inflow,
        house.to_account_info().lamports(),
        user_account.total_amount,
    )?;

    transfer_lamports(
        &house.to_account_info(),
        &user_info,
        weighted_lamports,
        &anchor_lang::solana_program::system_program::id(),
    )?;
}
Ok(())
}
fn calculate_streak_cap(ref_count: u8, streak: u64) -> u64 {
    let streak_cap = 100 * (1_u64.saturating_sub(ref_count as u64)) / 1_u64;
    let min_streak_cap = (streak as f64 / 0.4).ceil() as u64;
    streak_cap.max(min_streak_cap)
}

fn calculate_winner_reward(snapshot: u64, streak: u64, streak_cap: u64) -> u64 {
    let base_winner_percentage = 50_f64;
    let streak_bonus_percentage = (streak as f64 / streak_cap as f64) * base_winner_percentage;
    (snapshot.div_ceil(100) as f64 * (base_winner_percentage + streak_bonus_percentage)).round() as u64
}

fn process_referral_rewards<'info>(
    referrals: &[AccountInfo<'info>],
    house: &mut House,
) -> Result<()> {
    let mut prev_referral = None;
    for remaining_account in referrals.iter() {
        let remaining_user = User::try_from_slice(&remaining_account.data.borrow())?;
        let referral = remaining_user.referral;

        let referral_account = remaining_account.to_account_info();
        if prev_referral.map_or(true, |prev| !cmp_pubkeys(&prev, &referral_account.key)) {
            transfer_lamports(
                &house.to_account_info(),
                &referral_account,
                house.to_account_info().lamports().div_ceil(20),
                &anchor_lang::solana_program::system_program::id(),
            )?;
        }
        prev_referral = Some(referral_account.key);
    }
    Ok(())
}

fn calculate_weighted_lamports(
    house_lamports: u64,
    house_total_amount: u64,
    user_total_amount: u64,
) -> Result<u64> {
    const DECIMAL_PLACES: u64 = 9;
    const SCALE: u64 = 10u64.pow(DECIMAL_PLACES as u32);

    if house_total_amount == 0 {
        return Err(ProgramError::AccountBorrowFailed.into());
    }

    house_lamports
        .checked_mul(user_total_amount)
        .and_then(|product| product.checked_mul(SCALE))
        .and_then(|scaled_product| scaled_product.checked_div(house_total_amount))
        .and_then(|result| result.checked_div(SCALE))
        .ok_or(ProgramError::ArithmeticOverflow.into())
}

fn transfer_lamports<'info>(
    from_account: &AccountInfo<'info>,
    to_account: &AccountInfo<'info>,
    amount_of_lamports: u64,
    system_program: &Pubkey,
) -> Result<()> {
    if **from_account.try_borrow_lamports()? < amount_of_lamports {
        return Err(ProgramError::InsufficientFunds.into());
    }

    **from_account.try_borrow_mut_lamports()? -= amount_of_lamports;
    **to_account.try_borrow_mut_lamports()? += amount_of_lamports;

    Ok(())
}

impl User {
    fn update_commit(
        &mut self,
        amount: u64,
        user: &Signer,
        referral: &AccountInfo,
    ) -> Result<()> {
        self.amount = amount;
        self.total_amount += amount;
        assert_eq!(self.state, GameState::Ready, "User state is not ready");
        self.last_play = Clock::get()?.unix_timestamp;
        self.state = GameState::Committed;
        self.streak += 1;
        self.user = *user.to_account_info().key;
        self.referral = *referral.key;
        Ok(())
    }
}

impl House {
    fn update_inflow(&mut self, amount: u64) {
        self.total_inflow += amount;
    }

    fn update_win(
        &mut self,
        winner: Pubkey,
        won: u64,
        referrer_won: u64,
        referrer: Pubkey,
        referral_chain: u8,
    ) {
        self.recent_winner = winner;
        self.recent_won = won;
        self.recent_referrer_won = referrer_won;
        self.recent_referrer = referrer;
        self.total_won += won;
        self.total_wins += 1;
        self.recent_referral_chain = referral_chain;
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