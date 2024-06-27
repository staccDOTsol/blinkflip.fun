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
            &ctx.accounts.system_program,
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

fn calculate_streak_cap(ref_count