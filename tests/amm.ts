import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Amm } from "../target/types/amm";
import {
  createAssociatedTokenAccount,
  createMint,
  getAccount,
  getAssociatedTokenAddress,
  mintTo,
} from "@solana/spl-token";

describe("amm", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.amm as Program<Amm>;

  const provider = anchor.getProvider();

  const liquidity_provider = anchor.web3.Keypair.generate();
  const connection = provider.connection;
  const lp_mint_decimal: number = 9;
  async function getPda(seeds) {
    const [pda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      seeds,
      program.programId
    );
    return { pda, bump };
  }

  async function generateTokenMint() {
    const mint = await createMint(
      connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      9
    );
    return mint;
  }

  async function createBaseAndPCMint() {
    const base_mint = await generateTokenMint();
    const pc_mint = await generateTokenMint();

    return { base_mint, pc_mint };
  }

  async function prepareInitalizeLiquidityPool(
    base_mint_to_amount: number,
    pc_mint_to_amount: number
  ) {
    const { base_mint, pc_mint } = await createBaseAndPCMint();
    const liquidityProviderBaseTokenAta = await getAssociatedTokenAddress(
      base_mint,
      liquidity_provider.publicKey
    );
    const liquidityProviderPCTokenAta = await getAssociatedTokenAddress(
      pc_mint,
      liquidity_provider.publicKey
    );
    await createAssociatedTokenAccount(
      connection,
      provider.wallet.payer,
      base_mint,
      liquidity_provider.publicKey
    );

    await createAssociatedTokenAccount(
      connection,
      provider.wallet.payer,
      pc_mint,
      liquidity_provider.publicKey
    );
    await mintTo(
      connection,
      provider.wallet.payer,
      base_mint,
      liquidityProviderBaseTokenAta,
      provider.wallet.payer,
      base_mint_to_amount
    );
    await mintTo(
      connection,
      provider.wallet.payer,
      pc_mint,
      liquidityProviderPCTokenAta,
      provider.wallet.payer,
      base_mint_to_amount
    );
    const base_mint_amount = new anchor.BN(base_mint_to_amount);
    const pc_mint_amount = new anchor.BN(pc_mint_to_amount);
    const { pda: amm_pda } = await getPda([Buffer.from("amm_pda")]);
    const { pda: base_token_vault } = await getPda([
      Buffer.from("base_token_vault"),
      base_mint.toBuffer(),
    ]);
    const { pda: pc_token_vault } = await getPda([
      Buffer.from("pc_token_vault"),
      pc_mint.toBuffer(),
    ]);

    const { pda: lp_token_mint } = await getPda([
      Buffer.from("lp_mint"),
      base_mint.toBuffer(),
      pc_mint.toBuffer(),
      amm_pda.toBuffer(),
    ]);
    const { pda: liquidity_provider_lp_token_ata } = await getPda([
      Buffer.from("lp_token_ata"),
      liquidity_provider.publicKey.toBuffer(),
      amm_pda.toBuffer(),
    ]);

    return {
      amm_pda,
      base_token_vault,
      pc_token_vault,
      lp_token_mint,
      liquidity_provider_lp_token_ata,
      liquidityProviderPCTokenAta,
      liquidityProviderBaseTokenAta,
      base_mint,
      pc_mint,
      base_mint_amount,
      pc_mint_amount,
    };
  }
  before(async () => {
    const airdropSig = await provider.connection.requestAirdrop(
      liquidity_provider.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 100
    );
    await provider.connection.confirmTransaction(airdropSig);
  });
  it("Initialize liquidity pool", async () => {
    const {
      amm_pda,
      base_token_vault,
      pc_token_vault,
      lp_token_mint,
      liquidity_provider_lp_token_ata,
      liquidityProviderPCTokenAta,
      liquidityProviderBaseTokenAta,
      base_mint,
      pc_mint,
      base_mint_amount,
      pc_mint_amount,
    } = await prepareInitalizeLiquidityPool(2_000_000_000, 1_000_000_000);

    try {
      await program.methods
        .initializeLiquidity(
          lp_mint_decimal,
          base_mint,
          pc_mint,
          base_mint_amount,
          pc_mint_amount
        )
        .accounts({
          liquidityProvider: liquidity_provider.publicKey,
          ammPda: amm_pda,
          baseTokenVault: base_token_vault,
          pcTokenVault: pc_token_vault,
          lpTokenMint: lp_token_mint,
          liquidityProviderLpTokenAta: liquidity_provider_lp_token_ata,
          baseTokenMint: base_mint,
          pcTokenMint: pc_mint,
          liquidityProviderBaseTokenAta: liquidityProviderBaseTokenAta,
          liquidityProviderPcTokenAta: liquidityProviderPCTokenAta,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([liquidity_provider])
        .rpc();
    } catch (err) {
      console.log("Error");
      console.log(err);
    }

    const baseTokenAccount = await getAccount(
      provider.connection,
      base_token_vault
    );
    const pcTokenAccount = await getAccount(
      provider.connection,
      pc_token_vault
    );
    console.log("BaseTokenVault:");
    console.log(Number(baseTokenAccount.amount));
    console.log("PcTokenVault:");
    console.log(Number(pcTokenAccount.amount));
  });
});
