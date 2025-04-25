import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ContractsSolana } from "../target/types/contracts_solana";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { assert } from "chai";

describe("cross-chain-intents", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.ContractsSolana as Program<ContractsSolana>;
  const wallet = provider.wallet as anchor.Wallet;

  // Generate a new keypair for mint
  const mintKeypair = Keypair.generate();
  let userTokenAccount: PublicKey;
  let programTokenAccount: PublicKey;
  let gatewayTokenAccount: PublicKey;
  let statePda: PublicKey;
  let stateBump: number;
  let whitelistEntry: PublicKey;

  const gatewayAddress = Keypair.generate().publicKey;
  const routerAddress = Buffer.from("1234567890123456789012345678901234567890".substring(0, 40), "hex");

  before(async () => {
    // Find the state PDA
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("cross-chain-state")],
      program.programId
    );
    statePda = pda;
    stateBump = bump;

    // For testing purposes, we'll create a mock whitelist entry
    whitelistEntry = Keypair.generate().publicKey;
  });

  it("Initialize the cross-chain state", async () => {
    const tx = await program.methods
      .initialize(gatewayAddress, Array.from(routerAddress))
      .accounts({
        payer: wallet.publicKey,
        state: statePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log("Initialize transaction signature", tx);

    // Verify the state was initialized correctly
    const stateAccount = await program.account.crossChainState.fetch(statePda);
    assert.equal(stateAccount.intentCounter.toString(), "0");
    assert.deepEqual(stateAccount.gateway, gatewayAddress);
    
    // Convert Uint8Array to regular array for comparison
    const routerFromAccount = Array.from(stateAccount.router);
    assert.deepEqual(routerFromAccount, Array.from(routerAddress));
  });

  it("Get next intent ID", async () => {
    const salt = new anchor.BN(12345);
    
    // Call the getNextIntentId instruction
    const result = await program.methods
      .getNextIntentId(salt)
      .accounts({
        state: statePda,
      })
      .view();
    
    console.log("Next Intent ID:", Buffer.from(result).toString('hex'));
    assert.ok(result.length === 32, "Intent ID should be 32 bytes");
  });

  // Note: The actual token transfer and gateway call would require more
  // complex setup with mock accounts and CPI calls to the ZetaChain gateway.
  // This would be implementation-specific depending on the actual ZetaChain
  // gateway contract on Solana.
});
