use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{token::{self,CloseAccount, Mint, Token, TokenAccount, Transfer}};

declare_id!("9Vfg3sFgXTH79HfUaQDsT4vnXTVvPhJcXcmbnd1VzYV9");

#[program]
pub mod safe_exchange {
    use super::*;

    pub fn initialize_sol_token(
        ctx: Context<InitializeSolToken>,
        exchange_idx: u64,
        initializer_amount: u64,
        taker_amount: u64,
    ) -> Result<()> {
                
        let exchange_state = &mut ctx.accounts.exchange_state;
        let pubkey_vec = bs58::decode("So11111111111111111111111111111111111111112").into_vec().unwrap();
        exchange_state.exchange_idx = exchange_idx;
        exchange_state.initializer = ctx.accounts.initializer.key();
        exchange_state.initializer_mint = Pubkey::new(&pubkey_vec);
        exchange_state.initializer_amount = initializer_amount;
        exchange_state.taker = ctx.accounts.taker.key();
        exchange_state.taker_mint = ctx.accounts.taker_mint.key();
        exchange_state.taker_amount = taker_amount;

        let transfer_instruction = system_program::Transfer{
            from: ctx.accounts.initializer.to_account_info(),
            to: exchange_state.to_account_info(),
        };
        
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            transfer_instruction
        );
        system_program::transfer(cpi_context, initializer_amount)?;
        
        Ok(())
    }
    

    pub fn initialize_token_sol(
        ctx: Context<InitializeTokenSol>,
        exchange_idx: u64,
        initializer_amount: u64,
        taker_amount: u64,
    ) -> Result<()> {
                
        let exchange_state = &mut ctx.accounts.exchange_state;
        let pubkey_vec = bs58::decode("So11111111111111111111111111111111111111112").into_vec().unwrap();
        exchange_state.exchange_idx = exchange_idx;
        exchange_state.initializer = ctx.accounts.initializer.key();
        exchange_state.initializer_mint = ctx.accounts.initializer_mint.key();
        exchange_state.initializer_amount = initializer_amount;
        exchange_state.taker = ctx.accounts.taker.key();
        exchange_state.taker_mint = Pubkey::new(&pubkey_vec);
        exchange_state.taker_amount = taker_amount;

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
 
    pub fn initialize_token_token(
        ctx: Context<InitializeTokenToken>,
        exchange_idx: u64,
        initializer_amount: u64,
        taker_amount: u64,
    ) -> Result<()> {
                
        let exchange_state = &mut ctx.accounts.exchange_state;
        exchange_state.exchange_idx = exchange_idx;
        exchange_state.initializer = ctx.accounts.initializer.key();
        exchange_state.initializer_mint = ctx.accounts.initializer_mint.key();
        exchange_state.initializer_amount = initializer_amount;
        exchange_state.taker = ctx.accounts.taker.key();
        exchange_state.taker_mint = ctx.accounts.taker_mint.key();
        exchange_state.taker_amount = taker_amount;

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
    
    pub fn cancel_sol_token(ctx: Context<CancelSolToken>,exchange_idx: u64) -> Result<()> {
        
        let amount = ctx.accounts.exchange_state.initializer_amount;
        let seed0 = b"state";
        let seed1 = exchange_idx.to_le_bytes();
        let seed2 = ctx.accounts.initializer.key(); 
        let seed4 = ctx.accounts.taker.key();
        let seed5 = ctx.accounts.taker_mint.key();

        let (_exchange_state, _exchange_state_bump) =
        Pubkey::find_program_address(&[
            seed0.as_ref(),
            seed1.as_ref(),
            seed2.as_ref(),
            seed4.as_ref(),
            seed5.as_ref()
        ],    
        ctx.program_id);

        if ctx.accounts.exchange_state.key() != _exchange_state {
            panic!("exchange_state account is not correct.");
        }

        **ctx.accounts.exchange_state.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.initializer.try_borrow_mut_lamports()? += amount;
  
        Ok(())
    }

    
    pub fn cancel_token_sol(ctx: Context<CancelTokenSol>,exchange_idx: u64) -> Result<()> {
        
        let amount = ctx.accounts.exchange_state.initializer_amount;
        let bump = *ctx.bumps.get("exchange_state").unwrap();
        let seed0 = b"state";
        let seed1 = exchange_idx.to_le_bytes();
        let seed2 = ctx.accounts.initializer.key(); 
        let seed3 = ctx.accounts.initializer_mint.key();
        let seed4 = ctx.accounts.taker.key();
        let seed6 = [bump];

        let inner = vec![
            seed0.as_ref(),
            seed1.as_ref(),
            seed2.as_ref(),
            seed3.as_ref(),
            seed4.as_ref(),
            seed6.as_ref()
        ];

        let outer = vec![inner.as_slice()];

        let transfer_instruction = Transfer{
            from: ctx.accounts.escrow_wallet.to_account_info(),
            to: ctx.accounts.initializer_withdraw_token_account.to_account_info(),
            authority: ctx.accounts.exchange_state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );
        token::transfer(cpi_ctx, amount)?;

        let cpi_accounts = CloseAccount{
            account: ctx.accounts.escrow_wallet.to_account_info(),
            destination: ctx.accounts.initializer.to_account_info(),
            authority: ctx.accounts.exchange_state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            outer.as_slice(),
        );
        token::close_account(cpi_ctx)?;

        Ok(())
    }

    pub fn cancel_token_token(ctx: Context<CancelTokenToken>,exchange_idx: u64) -> Result<()> {
        
        let amount = ctx.accounts.exchange_state.initializer_amount;
        let bump = *ctx.bumps.get("exchange_state").unwrap();
        let seed0 = b"state";
        let seed1 = exchange_idx.to_le_bytes();
        let seed2 = ctx.accounts.initializer.key(); 
        let seed3 = ctx.accounts.initializer_mint.key();
        let seed4 = ctx.accounts.taker.key();
        let seed5 = ctx.accounts.taker_mint.key();
        let seed6 = [bump];

        let inner = vec![
            seed0.as_ref(),
            seed1.as_ref(),
            seed2.as_ref(),
            seed3.as_ref(),
            seed4.as_ref(),
            seed5.as_ref(),
            seed6.as_ref()
        ];

        let outer = vec![inner.as_slice()];

        let transfer_instruction = Transfer{
            from: ctx.accounts.escrow_wallet.to_account_info(),
            to: ctx.accounts.initializer_withdraw_token_account.to_account_info(),
            authority: ctx.accounts.exchange_state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );
        token::transfer(cpi_ctx, amount)?;

        let cpi_accounts = CloseAccount{
            account: ctx.accounts.escrow_wallet.to_account_info(),
            destination: ctx.accounts.initializer.to_account_info(),
            authority: ctx.accounts.exchange_state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            outer.as_slice(),
        );
        token::close_account(cpi_ctx)?;

        Ok(())
    }
   
    pub fn exchange_sol_token(ctx: Context<ExchangeSolToken>,exchange_idx:u64) -> Result<()> {
        let taker_amount = ctx.accounts.exchange_state.taker_amount;
        let initializer_amount = ctx.accounts.exchange_state.initializer_amount;

        let seed0 = b"state";
        let seed1 = exchange_idx.to_le_bytes();
        let seed2 = ctx.accounts.initializer.key(); 
        let seed4 = ctx.accounts.taker.key();
        let seed5 = ctx.accounts.taker_mint.key();

        let (_exchange_state, _exchange_state_bump) =
        Pubkey::find_program_address(&[
            seed0.as_ref(),
            seed1.as_ref(),
            seed2.as_ref(),
            seed4.as_ref(),
            seed5.as_ref()
        ],    
        ctx.program_id);

        if ctx.accounts.exchange_state.key() != _exchange_state {
            panic!("exchange_state account is not correct.");
        }

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

        **ctx.accounts.exchange_state.to_account_info().try_borrow_mut_lamports()? -= initializer_amount;
        **ctx.accounts.taker.try_borrow_mut_lamports()? += initializer_amount;

        Ok(())
    }

    pub fn exchange_token_sol(ctx: Context<ExchangeTokenSol>,exchange_idx:u64) -> Result<()> {
        let taker_amount = ctx.accounts.exchange_state.taker_amount;
        let initializer_amount = ctx.accounts.exchange_state.initializer_amount;

        let bump = *ctx.bumps.get("exchange_state").unwrap();
        let seed0 = b"state";
        let seed1 = exchange_idx.to_le_bytes();
        let seed2 = ctx.accounts.initializer.key(); 
        let seed3 = ctx.accounts.initializer_mint.key();
        let seed4 = ctx.accounts.taker.key();
        let seed6 = [bump];

        let inner = vec![
            seed0.as_ref(),
            seed1.as_ref(),
            seed2.as_ref(),
            seed3.as_ref(),
            seed4.as_ref(),
            seed6.as_ref()
        ];

        let outer = vec![inner.as_slice()];

        let transfer_instruction = system_program::Transfer{
            from: ctx.accounts.taker.to_account_info(),
            to: ctx.accounts.initializer.to_account_info(),
        };
        
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            transfer_instruction
        );
        system_program::transfer(cpi_context, taker_amount)?;

        let transfer_instruction = Transfer{
            from: ctx.accounts.escrow_wallet.to_account_info(),
            to: ctx.accounts.taker_receive_token_account.to_account_info(),
            authority: ctx.accounts.exchange_state.to_account_info(),
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
            authority: ctx.accounts.exchange_state.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            outer.as_slice(),
        );
        token::close_account(cpi_ctx)?;

        Ok(())
    } 

    pub fn exchange_token_token(ctx: Context<ExchangeTokenToken>,exchange_idx:u64) -> Result<()> {
        let taker_amount = ctx.accounts.exchange_state.taker_amount;
        let initializer_amount = ctx.accounts.exchange_state.initializer_amount;

        let bump = *ctx.bumps.get("exchange_state").unwrap();
        let seed0 = b"state";
        let seed1 = exchange_idx.to_le_bytes();
        let seed2 = ctx.accounts.initializer.key(); 
        let seed3 = ctx.accounts.initializer_mint.key();
        let seed4 = ctx.accounts.taker.key();
        let seed5 = ctx.accounts.taker_mint.key();
        let seed6 = [bump];

        let inner = vec![
            seed0.as_ref(),
            seed1.as_ref(),
            seed2.as_ref(),
            seed3.as_ref(),
            seed4.as_ref(),
            seed5.as_ref(),
            seed6.as_ref()
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
            authority: ctx.accounts.exchange_state.to_account_info(),
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
            authority: ctx.accounts.exchange_state.to_account_info(),
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
pub struct InitializeSolToken<'info> {
    #[account(mut)]
    initializer: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    taker: AccountInfo<'info>,
    taker_mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [b"state".as_ref(), 
            exchange_idx.to_le_bytes().as_ref(),
            initializer.key().as_ref(),
            taker.key().as_ref(),
            taker_mint.key().as_ref(),
        ],
        bump,
        payer = initializer,
        space = ExchangeState::space(),
    )]
    exchange_state: Box<Account<'info, ExchangeState>>,

    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
#[instruction(exchange_idx: u64, initializer_amount: u64, taker_amount: u64)]
pub struct InitializeTokenSol<'info> {
    #[account(mut)]
    initializer: Signer<'info>,
    initializer_mint: Account<'info, Mint>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    taker: AccountInfo<'info>,
    #[account(
        mut,
        constraint = initializer_withdraw_token_account.amount >= initializer_amount,
        constraint = initializer_withdraw_token_account.mint == initializer_mint.key(),
        constraint = initializer_withdraw_token_account.owner == initializer.key(),

    )]
    initializer_withdraw_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        seeds = [b"state".as_ref(), 
            exchange_idx.to_le_bytes().as_ref(),
            initializer.key().as_ref(),
            initializer_mint.key().as_ref(),
            taker.key().as_ref(),
        ],
        bump,
        payer = initializer,
        space = ExchangeState::space(),
    )]
    exchange_state: Box<Account<'info, ExchangeState>>,

    #[account(
        init,
        seeds = [b"wallet".as_ref(), 
            exchange_idx.to_le_bytes().as_ref(),
            initializer.key().as_ref(),
            initializer_mint.key().as_ref(),
            taker.key().as_ref(),
        ],
        bump,
        payer = initializer,
        token::mint = initializer_mint,
        token::authority = exchange_state,
    )]
    escrow_wallet: Account<'info, TokenAccount>,
    
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    token_program: Program<'info, Token>,
     
}
 
#[derive(Accounts)]
#[instruction(exchange_idx: u64, initializer_amount: u64, taker_amount: u64)]
pub struct InitializeTokenToken<'info> {
    #[account(mut)]
    initializer: Signer<'info>,
    initializer_mint: Account<'info, Mint>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    taker: AccountInfo<'info>,
    taker_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = initializer_withdraw_token_account.amount >= initializer_amount,
        constraint = initializer_withdraw_token_account.mint == initializer_mint.key(),
        constraint = initializer_withdraw_token_account.owner == initializer.key(),

    )]
    initializer_withdraw_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        seeds = [b"state".as_ref(), 
            exchange_idx.to_le_bytes().as_ref(),
            initializer.key().as_ref(),
            initializer_mint.key().as_ref(),
            taker.key().as_ref(),
            taker_mint.key().as_ref(),
        ],
        bump,
        payer = initializer,
        space = ExchangeState::space(),
    )]
    exchange_state: Box<Account<'info, ExchangeState>>,

    #[account(
        init,
        seeds = [b"wallet".as_ref(), 
            exchange_idx.to_le_bytes().as_ref(),
            initializer.key().as_ref(),
            initializer_mint.key().as_ref(),
            taker.key().as_ref(),
            taker_mint.key().as_ref(),
        ],
        bump,
        payer = initializer,
        token::mint = initializer_mint,
        token::authority = exchange_state,
    )]
    escrow_wallet: Account<'info, TokenAccount>,
    
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    token_program: Program<'info, Token>,
}


#[derive(Accounts)]
#[instruction(exchange_idx: u64)]
pub struct CancelSolToken<'info> {
    #[account(mut)]
    initializer: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    taker: AccountInfo<'info>,
    taker_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds=[b"state".as_ref(), 
               exchange_idx.to_le_bytes().as_ref(),
               initializer.key().as_ref(),
               taker.key().as_ref(),
               taker_mint.key().as_ref()],
        bump,
        has_one = initializer,
        has_one = taker,
        has_one = taker_mint,
        close = initializer,
    )]
    exchange_state: Box<Account<'info, ExchangeState>>,
     /// CHECK: This is not dangerous because we don't read or write from this account
    system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(exchange_idx: u64)]
pub struct CancelTokenSol<'info> {
    #[account(mut)]
    initializer: Signer<'info>,
    initializer_mint: Account<'info, Mint>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    taker: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = initializer_withdraw_token_account.owner == initializer.key(),
        constraint = initializer_withdraw_token_account.mint == initializer_mint.key(),
    )]
    initializer_withdraw_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds=[b"state".as_ref(), 
               exchange_idx.to_le_bytes().as_ref(),
               initializer.key().as_ref(),
               initializer_mint.key().as_ref(),
               taker.key().as_ref()],
        bump,
        has_one = initializer,
        has_one = initializer_mint,
        has_one = taker,
        close = initializer,
    )]
    exchange_state: Box<Account<'info, ExchangeState>>,
    #[account(
        mut,
        seeds=[b"wallet".as_ref(), 
               exchange_idx.to_le_bytes().as_ref(),
               initializer.key().as_ref(),
               initializer_mint.key().as_ref(),
               taker.key().as_ref()],
        bump,
        constraint = escrow_wallet.mint == initializer_mint.key(),
    )]
    escrow_wallet: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    token_program: Program<'info, Token>,
}


#[derive(Accounts)]
#[instruction(exchange_idx: u64)]
pub struct CancelTokenToken<'info> {
    #[account(mut)]
    initializer: Signer<'info>,
    initializer_mint: Account<'info, Mint>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    taker: AccountInfo<'info>,
    taker_mint: Account<'info, Mint>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = initializer_withdraw_token_account.owner == initializer.key(),
        constraint = initializer_withdraw_token_account.mint == initializer_mint.key(),
    )]
    initializer_withdraw_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds=[b"state".as_ref(), 
               exchange_idx.to_le_bytes().as_ref(),
               initializer.key().as_ref(),
               initializer_mint.key().as_ref(),
               taker.key().as_ref(),
               taker_mint.key().as_ref()],
        bump,
        has_one = initializer,
        has_one = initializer_mint,
        has_one = taker,
        has_one = taker_mint,
        close = initializer,
    )]
    exchange_state: Box<Account<'info, ExchangeState>>,
    #[account(
        mut,
        seeds=[b"wallet".as_ref(), 
               exchange_idx.to_le_bytes().as_ref(),
               initializer.key().as_ref(),
               initializer_mint.key().as_ref(),
               taker.key().as_ref(),
               taker_mint.key().as_ref()],
        bump,
        constraint = escrow_wallet.mint == initializer_mint.key(),
    )]
    escrow_wallet: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    token_program: Program<'info, Token>,
}


#[derive(Accounts)]
#[instruction(exchange_idx: u64)]
pub struct ExchangeSolToken<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    initializer: AccountInfo<'info>,
    #[account(mut)]
    taker: Signer<'info>,
    taker_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = initializer_receive_token_account.owner == initializer.key(),
        constraint = initializer_receive_token_account.mint == taker_mint.key(),
    )]
    initializer_receive_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = taker_withdraw_token_account.amount >= exchange_state.taker_amount,
        constraint = taker_withdraw_token_account.owner == taker.key(),
        constraint = taker_withdraw_token_account.mint == taker_mint.key(),
    )]
    taker_withdraw_token_account: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        seeds=[b"state".as_ref(), 
               exchange_idx.to_le_bytes().as_ref(),
               initializer.key().as_ref(),
               taker.key().as_ref(),
               taker_mint.key().as_ref()],
        bump,
        has_one = initializer,
        has_one = taker,
        has_one = taker_mint,
        close = initializer,
    )]
    exchange_state: Box<Account<'info, ExchangeState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(exchange_idx: u64)]
pub struct ExchangeTokenSol<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    initializer: AccountInfo<'info>,
    initializer_mint: Account<'info, Mint>,
    #[account(mut)]
    taker: Signer<'info>,
    #[account(
        mut,
        constraint = taker_receive_token_account.owner == taker.key(),
        constraint = taker_receive_token_account.mint == initializer_mint.key(),
    )]
    taker_receive_token_account: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        seeds=[b"state".as_ref(), 
               exchange_idx.to_le_bytes().as_ref(),
               initializer.key().as_ref(),
               initializer_mint.key().as_ref(),
               taker.key().as_ref()],
        bump,
        has_one = initializer,
        has_one = initializer_mint,
        has_one = taker,
        close = initializer,
    )]
    pub exchange_state: Box<Account<'info, ExchangeState>>,
    #[account(
        mut,
        seeds=[b"wallet".as_ref(), 
               exchange_idx.to_le_bytes().as_ref(),
               initializer.key().as_ref(),
               initializer_mint.key().as_ref(),
               taker.key().as_ref()],
        bump,
        constraint = escrow_wallet.mint == initializer_mint.key(),
    )]
    pub escrow_wallet: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    system_program: Program<'info, System>,
}



#[derive(Accounts)]
#[instruction(exchange_idx: u64)]
pub struct ExchangeTokenToken<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    initializer: AccountInfo<'info>,
    initializer_mint: Account<'info, Mint>,
    #[account(mut)]
    taker: Signer<'info>,
    taker_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = initializer_receive_token_account.owner == initializer.key(),
        constraint = initializer_receive_token_account.mint == taker_mint.key(),
    )]
    initializer_receive_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = taker_withdraw_token_account.amount >= exchange_state.taker_amount,
        constraint = taker_withdraw_token_account.owner == taker.key(),
        constraint = taker_withdraw_token_account.mint == taker_mint.key(),
    )]
    taker_withdraw_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = taker_receive_token_account.owner == taker.key(),
        constraint = taker_receive_token_account.mint == initializer_mint.key(),
    )]
    taker_receive_token_account: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        seeds=[b"state".as_ref(), 
               exchange_idx.to_le_bytes().as_ref(),
               initializer.key().as_ref(),
               initializer_mint.key().as_ref(),
               taker.key().as_ref(),
               taker_mint.key().as_ref()],
        bump,
        has_one = initializer,
        has_one = initializer_mint,
        has_one = taker,
        has_one = taker_mint,
        close = initializer,
    )]
    pub exchange_state: Box<Account<'info, ExchangeState>>,
    #[account(
        mut,
        seeds=[b"wallet".as_ref(), 
               exchange_idx.to_le_bytes().as_ref(),
               initializer.key().as_ref(),
               initializer_mint.key().as_ref(),
               taker.key().as_ref(),
               taker_mint.key().as_ref()],
        bump,
        constraint = escrow_wallet.mint == initializer_mint.key(),
    )]
    pub escrow_wallet: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}


#[account]
#[derive(Default)]
pub struct ExchangeState {
    pub exchange_idx:u64,
    pub initializer: Pubkey,
    pub taker: Pubkey,
    pub initializer_mint: Pubkey,
    pub taker_mint: Pubkey,
    pub initializer_amount: u64,
    pub taker_amount: u64,
}

impl ExchangeState {
    pub fn space() -> usize {
        8 + 4*32 + 3*8
    }
}
