use anchor_lang::prelude::*;

declare_id!("26zEAde3YAxwhy8mkyshsx7FhyfNJKHzU7cVLkdt895u");

#[program]
pub mod contracts_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
