use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use solana_program::hash::{hash, Hash};
use solana_program::pubkey::Pubkey;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::system_program;

declare_id!("26zEAde3YAxwhy8mkyshsx7FhyfNJKHzU7cVLkdt895u");

// Mock implementation of the ZetaChain Gateway
#[derive(Clone)]
pub struct GatewayMock;

impl GatewayMock {
    pub fn deposit_spl_token_and_call(
        ctx: Context<DepositSplToken>,
        amount: u64,
        receiver: [u8; 20],
        message: Vec<u8>,
        revert_options: Option<RevertOptions>,
    ) -> Result<()> {
        // Mock implementation of handling the deposit call
        msg!("Calling ZetaChain gateway deposit_spl_token_and_call");
        msg!("Amount: {}", amount);
        msg!("Receiver: {:?}", receiver);
        msg!("Message length: {}", message.len());
        
        // In the actual implementation, this would call handle_spl_with_call
        // instructions::deposit::handle_spl_with_call(
        //     ctx,
        //     amount,
        //     receiver,
        //     message,
        //     revert_options,
        //     DEPOSIT_FEE,
        // )
        
        Ok(())
    }
}

// Define the DepositSplToken context
#[derive(Accounts)]
pub struct DepositSplToken<'info> {
    /// The account of the signer making the deposit.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Gateway PDA.
    #[account(mut)]
    pub pda: AccountInfo<'info>,

    /// The whitelist entry account for the SPL token.
    pub whitelist_entry: AccountInfo<'info>,

    /// The mint account of the SPL token being deposited.
    pub mint_account: Account<'info, Mint>,

    /// The token program.
    pub token_program: Program<'info, Token>,

    /// The source token account owned by the signer.
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,

    /// The destination token account owned by the PDA.
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,

    /// The system program.
    pub system_program: Program<'info, System>,
}

// Struct containing revert options
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct RevertOptions {
    pub revert_address: Pubkey,
    pub abort_address: Pubkey,
    pub call_on_revert: bool,
    pub revert_message: Vec<u8>,
    pub on_revert_gas_limit: u64,
}

#[program]
pub mod contracts_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, gateway: Pubkey, router: [u8; 20]) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.intent_counter = 0;
        state.gateway = gateway;
        state.router = router;
        state.bump = *ctx.bumps.get("state").unwrap();
        Ok(())
    }

    pub fn initiate(
        ctx: Context<InitiateIntent>,
        amount: u64,
        target_chain: u64,
        receiver: Vec<u8>,
        tip: u64,
        salt: u64,
    ) -> Result<()> {
        // Cannot initiate a transfer to the current chain (Solana)
        require!(target_chain != 1, CrossChainError::InvalidTargetChain);

        // Calculate total amount to transfer (amount + tip)
        let total_amount = amount.checked_add(tip).ok_or(CrossChainError::ArithmeticError)?;

        // Get the state account
        let state = &mut ctx.accounts.state;
        
        // Generate intent ID
        let intent_id = compute_intent_id(state.intent_counter, salt);
        
        // Increment counter
        state.intent_counter = state.intent_counter.checked_add(1).ok_or(CrossChainError::ArithmeticError)?;

        // Create payload for crosschain transaction
        let payload = encode_intent_payload(intent_id, amount, tip, target_chain, &receiver);

        // Transfer tokens from user to the program account
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.program_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            total_amount,
        )?;

        // Create revert options for the ZetaChain gateway
        let revert_options = RevertOptions {
            revert_address: ctx.accounts.user.key(),
            abort_address: Pubkey::default(),
            call_on_revert: false,
            revert_message: Vec::new(),
            on_revert_gas_limit: 0,
        };

        // Create a DepositSplToken context from our accounts to pass to the gateway
        // In a real implementation, this would be a CPI call to the actual gateway
        let deposit_ctx = Context::new(
            &ctx.program_id,
            &DepositSplToken {
                signer: ctx.accounts.user.clone(),
                pda: ctx.accounts.program_token_account.to_account_info(),
                whitelist_entry: ctx.accounts.whitelist_entry.clone(),
                mint_account: ctx.accounts.mint.clone(),
                token_program: ctx.accounts.token_program.clone(),
                from: ctx.accounts.program_token_account.clone(),
                to: ctx.accounts.gateway_token_account.clone(),
                system_program: ctx.accounts.system_program.clone(),
            },
            &[],
            ctx.bumps.clone(),
        );
        
        // Call the gateway to handle the cross-chain transfer
        GatewayMock::deposit_spl_token_and_call(
            deposit_ctx,
            total_amount,
            state.router,
            payload,
            Some(revert_options),
        )?;

        // Emit event
        emit!(IntentInitiated {
            intent_id,
            asset: ctx.accounts.mint.key(),
            amount,
            target_chain,
            receiver: receiver.clone(),
            tip,
            salt,
        });

        Ok(())
    }

    pub fn get_next_intent_id(ctx: Context<GetNextIntentId>, salt: u64) -> Result<[u8; 32]> {
        let state = &ctx.accounts.state;
        let intent_id = compute_intent_id(state.intent_counter, salt);
        msg!("Next intent ID: {:?}", intent_id);
        Ok(intent_id)
    }
}

// Event emitted when a new intent is created
#[event]
pub struct IntentInitiated {
    #[index]
    pub intent_id: [u8; 32],
    #[index]
    pub asset: Pubkey,
    pub amount: u64,
    pub target_chain: u64,
    pub receiver: Vec<u8>,
    pub tip: u64,
    pub salt: u64,
}

#[account]
pub struct CrossChainState {
    pub intent_counter: u64,
    pub gateway: Pubkey,
    pub router: [u8; 20],
    pub bump: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        space = 8 + 8 + 32 + 20 + 1, // discriminator + intent_counter + gateway + router + bump
        seeds = [b"cross-chain-state"],
        bump
    )]
    pub state: Account<'info, CrossChainState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitiateIntent<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub gateway_token_account: Account<'info, TokenAccount>,
    
    pub mint: Account<'info, Mint>,
    
    pub whitelist_entry: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [b"cross-chain-state"],
        bump = state.bump
    )]
    pub state: Account<'info, CrossChainState>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetNextIntentId<'info> {
    #[account(
        seeds = [b"cross-chain-state"],
        bump = state.bump
    )]
    pub state: Account<'info, CrossChainState>,
}

// Helper function to compute a unique intent ID
pub fn compute_intent_id(counter: u64, salt: u64) -> [u8; 32] {
    let counter_bytes = counter.to_le_bytes();
    let salt_bytes = salt.to_le_bytes();
    
    // Concatenate counter and salt bytes
    let mut data = Vec::with_capacity(counter_bytes.len() + salt_bytes.len());
    data.extend_from_slice(&counter_bytes);
    data.extend_from_slice(&salt_bytes);
    
    // Hash the data to create a unique ID
    hash(&data).to_bytes()
}

// Helper function to encode intent payload for the ZetaChain gateway
pub fn encode_intent_payload(
    intent_id: [u8; 32],
    amount: u64,
    tip: u64,
    target_chain: u64,
    receiver: &[u8],
) -> Vec<u8> {
    let mut payload = Vec::new();
    
    // Add intent ID
    payload.extend_from_slice(&intent_id);
    
    // Add amount
    payload.extend_from_slice(&amount.to_le_bytes());
    
    // Add tip
    payload.extend_from_slice(&tip.to_le_bytes());
    
    // Add target chain
    payload.extend_from_slice(&target_chain.to_le_bytes());
    
    // Add receiver length and bytes
    let receiver_len = receiver.len() as u32;
    payload.extend_from_slice(&receiver_len.to_le_bytes());
    payload.extend_from_slice(receiver);
    
    payload
}

#[error_code]
pub enum CrossChainError {
    #[msg("Target chain cannot be the current chain")]
    InvalidTargetChain,
    #[msg("Arithmetic error")]
    ArithmeticError,
}
