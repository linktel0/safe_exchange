use anchor_lang::prelude::*;
use anchor_spl::{token::{self,CloseAccount, Mint, Token, TokenAccount, Transfer}};

declare_id!("DT6vGdjj2GKHJEnheWLaS1PKSoc7hwMSp5CmnPVUt6jC");

#[program]
pub mod safe_exchange {
    use super::*;

    pub fn initialize_token_token(
        ctx: Context<InitializeTokenToken>,
        exchange_idx: u64,
        initializer_amount: u64,
        taker_amount: u64,
    ) -> Result<()> {
                
        ctx.accounts.escrow_state.exchange_idx = exchange_idx;
        ctx.accounts.escrow_state.initializer = *ctx.accounts.initializer.key;
        ctx.accounts.escrow_state.initializer_mint = ctx
            .accounts
            .initializer_withdraw_token_account
            .mint;
        ctx.accounts.escrow_state.taker_mint = ctx
            .accounts
            .initializer_receive_token_account
            .mint;
        ctx.accounts.escrow_state.initializer_amount = initializer_amount;
        ctx.accounts.escrow_state.taker_amount = taker_amount;

       let transfer_instruction = Transfer{
            from: ctx.accounts.initializer_withdraw_token_account.to_account_info(),
            to: ctx.accounts.escrow_wallet.to_account_info(),
            authority: ctx.accounts.initializer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        token::transfer(cpi_ctx, initializer_amount)?;
         
        Ok(())
    }
    
    
    pub fn cancel_token_token(ctx: Context<CancelTokenToken>) -> Result<()> {
        
        let initializer_amount = ctx.accounts.escrow_state.initializer_amount;
        let seed0 = b"state";
        let seed1 = ctx.accounts.escrow_state.exchange_idx.to_le_bytes(); 
        let seed2 = ctx.accounts.initializer_withdraw_token_account.key();
        let seed3 = ctx.accounts.initializer_receive_token_account.key();
               
        let (_escrow_authority, _escrow_authority_bump) =
            Pubkey::find_program_address(&[
                seed0.as_ref(),
                seed1.as_ref(),
                seed2.as_ref(),
                seed3.as_ref()
            ],    
            ctx.program_id);

        let seed4 = [_escrow_authority_bump];

        let inner = vec![
            seed0.as_ref(),
            seed1.as_ref(),
            seed2.as_ref(),
            seed3.as_ref(),
            seed4.as_ref()
        ];

        let outer = vec![inner.as_slice()];

        let transfer_instruction = Transfer{
            from: ctx.accounts.escrow_wallet.to_account_info(),
            to: ctx.accounts.initializer_withdraw_token_account.to_account_info(),
            authority: ctx.accounts.escrow_state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );
        token::transfer(cpi_ctx, initializer_amount)?;

        let cpi_accounts = CloseAccount{
            account: ctx.accounts.escrow_wallet.to_account_info(),
            destination: ctx.accounts.initializer.to_account_info(),
            authority: ctx.accounts.escrow_state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            outer.as_slice(),
        );
        token::close_account(cpi_ctx)?;

        Ok(())
    }
   
    
    pub fn exchange_token_token(ctx: Context<ExchangeTokenToken>) -> Result<()> {
        let taker_amount = ctx.accounts.escrow_state.taker_amount;
        let initializer_amount = ctx.accounts.escrow_state.initializer_amount;

        let seed0 = b"state";
        let seed1 = ctx.accounts.escrow_state.exchange_idx.to_le_bytes(); 
        let seed2 = ctx.accounts.initializer_withdraw_token_account.key();
        let seed3 = ctx.accounts.initializer_receive_token_account.key();
               
        let (_escrow_authority, _escrow_authority_bump) =
            Pubkey::find_program_address(&[
                seed0.as_ref(),
                seed1.as_ref(),
                seed2.as_ref(),
                seed3.as_ref()
            ],    
            ctx.program_id);

        let seed4 = [_escrow_authority_bump];

        let inner = vec![
            seed0.as_ref(),
            seed1.as_ref(),
            seed2.as_ref(),
            seed3.as_ref(),
            seed4.as_ref()
        ];

        let outer = vec![inner.as_slice()];


        let transfer_instruction = Transfer{
            from: ctx.accounts.taker_withdraw_token_account.to_account_info(),
            to: ctx.accounts.initializer_receive_token_account.to_account_info(),
            authority: ctx.accounts.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        token::transfer(cpi_ctx, taker_amount)?;

        let transfer_instruction = Transfer{
            from: ctx.accounts.escrow_wallet.to_account_info(),
            to: ctx.accounts.taker_receive_token_account.to_account_info(),
            authority: ctx.accounts.escrow_state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );

        token::transfer(cpi_ctx, initializer_amount)?;

        let cpi_accounts = CloseAccount{
            account: ctx.accounts.escrow_wallet.to_account_info(),
            destination: ctx.accounts.initializer.to_account_info(),
            authority: ctx.accounts.escrow_state.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            outer.as_slice(),
        );
        token::close_account(cpi_ctx)?;

        Ok(())
    }
     
}



#[derive(Accounts)]
#[instruction(exchange_idx: u64, initializer_amount: u64, taker_amount: u64)]
pub struct InitializeTokenToken<'info> {
    #[account(mut)]
    initializer: Signer<'info>,
    initializer_mint: Account<'info, Mint>,
    taker_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = initializer_withdraw_token_account.amount >= initializer_amount,
        constraint = initializer_withdraw_token_account.mint == initializer_mint.key(),
        constraint = initializer_withdraw_token_account.owner == initializer.key(),

    )]
    initializer_withdraw_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = initializer_receive_token_account.mint == taker_mint.key(),
        constraint = initializer_receive_token_account.owner == initializer.key(),
    )]
    initializer_receive_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        seeds = [b"state".as_ref(), 
            exchange_idx.to_le_bytes().as_ref(),
            initializer_withdraw_token_account.key().as_ref(),
            initializer_receive_token_account.key().as_ref(),
        ],
        bump,
        payer = initializer,
        space = EscrowState::space(),
    )]
    pub escrow_state: Box<Account<'info, EscrowState>>,

    #[account(
        init,
        seeds = [b"wallet".as_ref(), 
            exchange_idx.to_le_bytes().as_ref(),
            initializer_withdraw_token_account.key().as_ref(),
            initializer_receive_token_account.key().as_ref(),
        ],
        bump,
        payer = initializer,
        token::mint = initializer_mint,
        token::authority = escrow_state,
    )]
    escrow_wallet: Account<'info, TokenAccount>,
    
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelTokenToken<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub initializer: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = initializer_withdraw_token_account.owner == *initializer.key,
        constraint = initializer_withdraw_token_account.mint == escrow_state.initializer_mint,
    )]
    pub initializer_withdraw_token_account: Account<'info, TokenAccount>,
    #[account()]
    pub initializer_receive_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = escrow_state.initializer == initializer.key(),
        close = initializer
    )]
    pub escrow_state: Box<Account<'info, EscrowState>>,
    #[account(
        mut,
        token::authority = escrow_state,
        //constraint = escrow_wallet.amount >= escrow_state.initializer_amount,
        constraint = escrow_wallet.mint == escrow_state.initializer_mint,
    )]
    pub escrow_wallet: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ExchangeTokenToken<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub taker: Signer<'info>,
    #[account(mut)]
    pub taker_withdraw_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub taker_receive_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_withdraw_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_receive_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub initializer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = escrow_state.taker_amount <= taker_withdraw_token_account.amount,
        constraint = escrow_state.initializer_amount <= escrow_wallet.amount,
        constraint = escrow_state.initializer == initializer.key(),
        close = initializer
    )]
    pub escrow_state: Box<Account<'info, EscrowState>>,
    #[account(mut)]
    pub escrow_wallet: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(Default)]
pub struct EscrowState {
    pub exchange_idx:u64,
    pub initializer: Pubkey,
    pub initializer_mint: Pubkey,
    pub taker_mint: Pubkey,
    pub initializer_amount: u64,
    pub taker_amount: u64,
}

impl EscrowState {
    pub fn space() -> usize {
        8 + 3*(32 + 8)
    }
}
