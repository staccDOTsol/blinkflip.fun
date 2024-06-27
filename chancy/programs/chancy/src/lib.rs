use anchor_lang::solana_program::address_lookup_table::{
    instruction::extend_lookup_table, state::AddressLookupTable,
};
use anchor_lang::{
    prelude::*,
    solana_program::{
        program::invoke, program_memory::sol_memcmp, system_instruction, sysvar::SysvarId,
    },
};
use std::{str::FromStr, vec};

declare_id!("76NigLJb5MPHMz4UyYHeAKR1v4Ck1SFrAkBjVKmbYJpA");

pub const BLOCK_HASHES: Pubkey = pubkey!("SysvarRecentB1ockHashes11111111111111111111");

#[program]
pub mod chancy {

    use super::*;

    pub fn commit(ctx: Context<Commit>, amount: u64) -> Result<()> {
        let user = &mut ctx.accounts.user_account;
        user.update_commit(amount, &ctx.accounts.user, &ctx.accounts.referral)?;

        let house = &mut ctx.accounts.house;
        house.update_inflow(amount);

        invoke(
            &system_instruction::transfer(
                ctx.accounts.user.to_account_info().key,
                house.to_account_info().key,
                amount,
            ),
            &[
                ctx.accounts.user.to_account_info().clone(),
                house.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;

        Ok(())
    }

    pub fn reveal<'info>(
        ctx: Context<'_, '_, 'info, 'info, Reveal<'info>>,
        ref_count: u8,
        lut_count: u8,
    ) -> Result<()> {
        msg!("Starting reveal function");
        let dev = &mut ctx.accounts.dev;
        let lookup_table_table = &mut ctx.accounts.lookup_table_table;
        msg!("Initialized dev, lookup_table_table, and lookup");

        let (referrals, luts, users) =
            parse_remaining_accounts(ctx.remaining_accounts, ref_count, lut_count);
        msg!(
            "Parsed remaining accounts: {} referrals, {} luts, {} users",
            referrals.len(),
            luts.len(),
            users.len()
        );

        let named_accounts = vec![
            *ctx.accounts.user_account.to_account_info().key,
            *ctx.accounts.house.to_account_info().key,
            *ctx.accounts.recent_blockhashes.to_account_info().key,
            *ctx.accounts.referral.to_account_info().key,
        ];
        msg!("Created named_accounts vector");

        update_lookup_tables(
            luts,
            lookup_table_table,
            dev.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            named_accounts,
        )?;
        msg!("Updated lookup tables");

        let user = &mut ctx.accounts.user_account;
        let house = &mut ctx.accounts.house;
        let recent_slothashes = &ctx.accounts.recent_blockhashes;
        msg!("Initialized user, house, and recent_slothashes");

        validate_blockhashes(recent_slothashes)?;
        msg!("Validated blockhashes");

        let (modded, rounded_amount) =
            calculate_modded_and_rounded_amount(recent_slothashes, user)?;
        msg!(
            "Calculated modded: {}, rounded_amount: {}",
            modded,
            rounded_amount
        );

        if modded <= rounded_amount {
            msg!("User won. Processing win...");
            process_win(
                user,
                house,
                ctx.accounts.user.to_account_info(),
                dev.to_account_info(),
                ctx.accounts.referral.to_account_info(),
                referrals,
                ref_count,
            )?;
            msg!("Win processed successfully");
        } else {
            msg!("User lost. Processing loss...");
            process_loss(users, house)?;
            msg!("Loss processed successfully");
        }
        user.state = GameState::Ready;
        Ok(())
    }
}


fn parse_remaining_accounts<'info>(
    remaining_accounts: &'info [AccountInfo<'info>],
    ref_count: u8,
    lut_count: u8,
) -> (Vec<AccountInfo<'info>>, Vec<AccountInfo<'info>>, Vec<AccountInfo<'info>>) {
    let total_count = remaining_accounts.len();
    let referrals_end = total_count - lut_count as usize;
    let luts_start = referrals_end;
    let users_end = total_count - lut_count as usize - ref_count as usize;

    let referrals = remaining_accounts[users_end..referrals_end].to_vec();
    let luts = remaining_accounts[luts_start..total_count].to_vec();
    let users = remaining_accounts[0..users_end].to_vec();

    (referrals, luts, users)
}

fn update_lookup_tables<'info>(
    luts: Vec<AccountInfo<'info>>,
    lookup_table_table: &mut Account<'info, LookupTableTable>,
    dev: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    named_accounts: Vec<Pubkey>,
) -> Result<()> {
    let mut pubkeys_to_add: Vec<Pubkey> = named_accounts;
    for (index, ai) in luts.iter().enumerate() {
        let data = ai.try_borrow_data()?;
        let lookup_table = AddressLookupTable::deserialize(&data).unwrap();
        if index == luts.len() {
            
        if lookup_table.addresses.len() < 255 - pubkeys_to_add.len() {
            msg!(
                "Extending lookup table with {} pubkeys",
                pubkeys_to_add.len()
            );
            let extend_instruction =
                extend_lookup_table(*ai.key, *dev.key, Some(*dev.key), pubkeys_to_add.clone());
            msg!("Created extend instruction");
            invoke(
                &extend_instruction,
                &[
                    ai.clone(),
                    dev.clone(),
                    system_program.clone(),
                ],
            )?;
        }
        }
        pubkeys_to_add.retain(|pubkey| !lookup_table.addresses.contains(pubkey));
    }


    for ai in luts.clone() {
        msg!("Processing lookup table");

        let lookup_table_table_info = lookup_table_table.to_account_info();
        if !lookup_table_table_info.data.borrow().windows(32).any(|window| window == ai.key.to_bytes()) {
            msg!("Adding lookup table to lookup_table_table");
            msg!("Resizing lookup_table_table account");
            let new_size = lookup_table_table_info.data.borrow().len() + 32;
            let rent = Rent::get()?;
            let new_minimum_balance = rent.minimum_balance(new_size);

            msg!("Transferring lamports to cover reallocation");
            let lamports_diff = new_minimum_balance.saturating_sub(lookup_table_table_info.lamports());
            invoke(
                &system_instruction::transfer(dev.key, lookup_table_table_info.key, lamports_diff),
                &[
                    dev.clone(),
                    lookup_table_table_info.clone(),
                    system_program.clone(),
                ],
            )?;

            msg!("Reallocating lookup_table_table account");
            lookup_table_table_info.realloc(new_size, false)?;
            lookup_table_table.lookup_tables.push(*ai.key);
        }
    }
    Ok(())
}

fn validate_blockhashes(recent_slothashes: &UncheckedAccount) -> Result<()> {
    if cmp_pubkeys(recent_slothashes.key, &BLOCK_HASHES) {
        msg!("recent_blockhashes is deprecated and will break soon");
    }
    if !cmp_pubkeys(recent_slothashes.key, &SlotHashes::id())
        && !cmp_pubkeys(recent_slothashes.key, &BLOCK_HASHES)
    {
        return Err(ErrorCode::InvalidBlockhashes.into());
    }
    Ok(())
}

fn calculate_modded_and_rounded_amount(
    recent_slothashes: &UncheckedAccount,
    user: &User,
) -> Result<(f64, f64)> {
    let data = recent_slothashes.try_borrow_data()?;
    let clock = Clock::get()?;
    let timestamp = clock.unix_timestamp;

    let modded: usize = timestamp.checked_rem(16).ok_or(ErrorCode::InvalidModulus)? as usize;

    let most_recent = arrayref::array_ref![data, modded, 8];
    let index = u64::from_le_bytes(*most_recent);
    let modded: f64 = index.checked_rem(1000).ok_or(ErrorCode::InvalidModulus)? as f64;

    let rounded_amount = (user.amount as f64 / 10f64.powi(6)).round() as f64 / 2.0 / 1000.0;
    drop(data);
    Ok((modded, rounded_amount))
}

fn process_win<'info>(
    user: &mut Account<'info, User>,
    house: &mut Account<'info, House>,
    user_account: AccountInfo<'info>,
    dev: AccountInfo<'info>,
    referral: AccountInfo<'info>,
    referrals: Vec<AccountInfo<'info>>,
    ref_count: u8,
) -> Result<()> {
    let streak_cap = calculate_streak_cap(ref_count, user.streak);
    if user.streak > streak_cap {
        msg!("Sorry! You're streaking too high! Back to 0 for you.");
        user.streak = 0;
    }
    let house_ai = house.to_account_info();
    let snapshot = house_ai.lamports();
    let winner_reward = calculate_winner_reward(snapshot, user.streak, streak_cap);

    house.update_win(
        *user_account.key,
        winner_reward,
        snapshot.div_ceil(10),
        *referral.key,
        ref_count,
    );

    transfer_lamports(&house_ai, &user_account, winner_reward)?;
    transfer_lamports(&house_ai, &dev, snapshot.div_ceil(10))?;
    transfer_lamports(&house_ai, &referral, snapshot.div_ceil(10))?;

    process_referral_rewards(referrals, house)?;

    user.streak = 0;
    Ok(())
}

fn process_loss<'info>(
    users: Vec<AccountInfo<'info>>,
    house: &mut Account<'info, House>,
) -> Result<()> {
    let house_ai = house.to_account_info();
    for user_ai in users {
        let user_info = user_ai.clone();
let user_account = User::deserialize(&mut &user_ai.try_borrow_data()?[..])?;
if house.total_inflow == 0 {
    house.total_inflow = 1_000_000_000;
}
if user_account.total_amount == 0 {
    continue;
}

        let weighted_lamports = calculate_weighted_lamports(
            house.total_inflow,
            house_ai.lamports(),
            user_account.total_amount,
        )?;

        transfer_lamports(&house_ai, &user_info, weighted_lamports)?;
        drop(user_account);
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
    (snapshot.div_ceil(100) as f64 * (base_winner_percentage + streak_bonus_percentage)).round()
        as u64
}

fn process_referral_rewards<'info>(
    referrals: Vec<AccountInfo<'info>>,
    house: &mut Account<'info, House>,
) -> Result<()> {
    let mut prev_referral = None;
    let house_ai = house.to_account_info();
    for remaining_account in referrals {
        let referral_account = remaining_account;
        if prev_referral.map_or(true, |prev| !cmp_pubkeys(&prev, referral_account.key)) {
            transfer_lamports(
                &house_ai,
                &referral_account,
                house_ai.lamports().div_ceil(20),
            )?;
        }
        prev_referral = Some(*referral_account.key);
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
    // Calculate the weighted lamports using checked arithmetic operations
    let scaled_user_amount = user_total_amount
        .checked_mul(SCALE)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let scaled_product = scaled_user_amount
        .checked_mul(house_lamports)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let result = scaled_product
        .checked_div(house_total_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    result
        .checked_div(SCALE)
        .ok_or(ProgramError::ArithmeticOverflow.into())
}

fn transfer_lamports<'info>(
    from_account: &AccountInfo<'info>,
    to_account: &AccountInfo<'info>,
    amount_of_lamports: u64,
) -> Result<()> {
    if **from_account.try_borrow_lamports()? < amount_of_lamports {
        return Err(ProgramError::InsufficientFunds.into());
    }

    **from_account.try_borrow_mut_lamports()? -= amount_of_lamports;
    **to_account.try_borrow_mut_lamports()? += amount_of_lamports;

    Ok(())
}

impl User {
    fn update_commit(&mut self, amount: u64, user: &Signer, referral: &AccountInfo) -> Result<()> {
        self.amount = amount;
        self.total_amount += amount;
        if self.state != GameState::Ready {
            return Err(ErrorCode::InvalidState.into());
        }
        self.last_play = Clock::get()?.unix_timestamp;
        self.state = GameState::Committed;
        self.streak += 1;
        self.user = *user.key;
        self.referral = *referral.key;
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
    pub system_program: Program<'info, System>,
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
    pub total_inflow: u64,
}

#[account]
pub struct User {
    pub referral: Pubkey,

    pub user: Pubkey,
    pub amount: u64,
    pub streak: u64,
    pub state: GameState,
    pub last_play: i64,
    pub total_amount: u64,
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
    InvalidBlockhashes,
}

fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), 32) == 0
}
