import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Amm } from "../target/types/amm";
import {
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  createMint,
  createSyncNativeInstruction,
  getAccount,
  getAssociatedTokenAddress,
  mintTo,
  NATIVE_MINT,
} from "@solana/spl-token";
import {
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { assert } from "chai";

describe("amm", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.amm as Program<Amm>;

  const provider = anchor.getProvider();

  const liquidityProvider = anchor.web3.Keypair.generate();
  const connection = provider.connection;
  const lpMintDecimal: number = 9;
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
    const baseMint = await generateTokenMint();
    const pcMint = await generateTokenMint();

    return { baseMint, pcMint };
  }

  async function prepareInitalizeLiquidityPool(
    baseMintToAmount: number,
    pcMintToAmount: number,
    ammPdaIndex: anchor.BN,
    isNativeBase: boolean = false,
    isNativePc: boolean = false
  ) {
    let baseMint;
    let pcMint;
    if (isNativeBase) {
      baseMint = NATIVE_MINT;
      ({ pcMint } = await createBaseAndPCMint());
    } else if (isNativePc) {
      pcMint = NATIVE_MINT;
      ({ baseMint } = await createBaseAndPCMint());
    } else {
      ({ baseMint, pcMint } = await createBaseAndPCMint());
    }

    const liquidityProviderBaseTokenAta = await getAssociatedTokenAddress(
      baseMint,
      liquidityProvider.publicKey
    );
    const liquidityProviderPCTokenAta = await getAssociatedTokenAddress(
      pcMint,
      liquidityProvider.publicKey
    );

    if (!isNativeBase) {
      await createAssociatedTokenAccount(
        connection,
        provider.wallet.payer,
        baseMint,
        liquidityProvider.publicKey
      );
      await mintTo(
        connection,
        provider.wallet.payer,
        baseMint,
        liquidityProviderBaseTokenAta,
        provider.wallet.payer,
        baseMintToAmount
      );
    }
    if (!isNativePc) {
      await createAssociatedTokenAccount(
        connection,
        provider.wallet.payer,
        pcMint,
        liquidityProvider.publicKey
      );
      await mintTo(
        connection,
        provider.wallet.payer,
        pcMint,
        liquidityProviderPCTokenAta,
        provider.wallet.payer,
        baseMintToAmount
      );
    }

    const baseMintAmount = new anchor.BN(baseMintToAmount);
    const pcMintAmount = new anchor.BN(pcMintToAmount);
    const { pda: ammPda } = await getPda([
      Buffer.from("amm_pda"),
      ammPdaIndex.toArrayLike(Buffer, "le", 8),
    ]);
    const { pda: baseTokenVault } = await getPda([
      Buffer.from("base_token_vault"),
      baseMint.toBuffer(),
    ]);
    const { pda: pcTokenVault } = await getPda([
      Buffer.from("pc_token_vault"),
      pcMint.toBuffer(),
    ]);

    const { pda: lpTokenMint } = await getPda([
      Buffer.from("lp_mint"),
      baseMint.toBuffer(),
      pcMint.toBuffer(),
      ammPda.toBuffer(),
    ]);
    const { pda: liquidityProviderLpTokenAta } = await getPda([
      Buffer.from("lp_token_ata"),
      liquidityProvider.publicKey.toBuffer(),
      ammPda.toBuffer(),
    ]);

    return {
      ammPda,
      baseTokenVault,
      pcTokenVault,
      lpTokenMint,
      liquidityProviderLpTokenAta,
      liquidityProviderPCTokenAta,
      liquidityProviderBaseTokenAta,
      baseMint,
      pcMint,
      baseMintAmount,
      pcMintAmount,
    };
  }
  async function wrapSol(
    wallet: anchor.web3.Keypair,
    associatedTokenAccount: anchor.web3.PublicKey
  ) {
    // const associatedTokenAccount = await getAssociatedTokenAddress(
    //   NATIVE_MINT,
    //   wallet.publicKey
    // );

    const wrapTransaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        associatedTokenAccount,
        wallet.publicKey,
        NATIVE_MINT
      ),
      SystemProgram.transfer({
        fromPubkey: wallet.publicKey,
        toPubkey: associatedTokenAccount,
        lamports: LAMPORTS_PER_SOL * 10,
      }),
      createSyncNativeInstruction(associatedTokenAccount)
    );
    await sendAndConfirmTransaction(connection, wrapTransaction, [wallet]);

    return associatedTokenAccount;
  }
  before(async () => {
    const airdropSig = await provider.connection.requestAirdrop(
      liquidityProvider.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 100
    );
    await provider.connection.confirmTransaction(airdropSig);
  });
  it("Initialize liquidity pool", async () => {
    const ammPdaIndex = new anchor.BN(1);
    const {
      ammPda,
      baseTokenVault,
      pcTokenVault,
      lpTokenMint,
      liquidityProviderLpTokenAta,
      liquidityProviderPCTokenAta,
      liquidityProviderBaseTokenAta,
      baseMint,
      pcMint,
      baseMintAmount,
      pcMintAmount,
    } = await prepareInitalizeLiquidityPool(
      2_000_000_000,
      1_000_000_000,
      ammPdaIndex
    );

    const lpTokenMintAmount = 414213562;
    let txSig;
    try {
      txSig = await program.methods
        .initializeLiquidity(
          lpMintDecimal,
          ammPdaIndex,
          baseMint,
          pcMint,
          baseMintAmount,
          pcMintAmount
        )
        .accounts({
          liquidityProvider: liquidityProvider.publicKey,
          ammPda: ammPda,
          baseTokenVault: baseTokenVault,
          pcTokenVault: pcTokenVault,
          lpTokenMint: lpTokenMint,
          liquidityProviderLpTokenAta: liquidityProviderLpTokenAta,
          baseTokenMint: baseMint,
          pcTokenMint: pcMint,
          liquidityProviderBaseTokenAta: liquidityProviderBaseTokenAta,
          liquidityProviderPcTokenAta: liquidityProviderPCTokenAta,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([liquidityProvider])
        .rpc({ commitment: "confirmed" });
    } catch (err) {
      console.log("Error");
      console.log(err);
    }

    const baseTokenAccount = await getAccount(
      provider.connection,
      baseTokenVault
    );
    const pcTokenAccount = await getAccount(provider.connection, pcTokenVault);
    const lpTokenAccount = await getAccount(
      provider.connection,
      liquidityProviderLpTokenAta
    );
    assert.equal(
      baseTokenAccount.amount.toString(),
      baseMintAmount.toString(),
      "Base token mint amount doesn't match"
    );

    assert.equal(
      lpTokenAccount.amount.toString(),
      lpTokenMintAmount.toString(),
      "Lp token mint amount doesn't match"
    );
    assert.equal(
      pcTokenAccount.amount.toString(),
      pcMintAmount.toString(),
      "Pc token mint amount doesn't match"
    );
    // Check emitted event
    // const listenerMyEvent = program.addEventListener(
    //   "InitializeLiquidityPoolEvent",
    //   (event, slot) => {
    //     console.log(`slot ${slot} event value ${event}`);
    //   }
    // );
    // await program.removeEventListener(listenerMyEvent);
    const tx = await provider.connection.getTransaction(txSig, {
      commitment: "confirmed",
    });
    const eventParser = new anchor.EventParser(
      program.programId,
      new anchor.BorshCoder(program.idl)
    );
    const events = eventParser.parseLogs(tx.meta.logMessages);
    let logEmitted = false;
    for (let event of events) {
      if (event.name == "initializeLiquidityPoolEvent") {
        logEmitted = true;
        assert.equal(
          event.data.liquidityProvider.toString(),
          liquidityProvider.publicKey.toString(),
          "Event liquidity provder should match with actual liquidity provder"
        );
        assert.equal(
          event.data.baseTokenMint.toString(),
          baseMint.toString(),
          "Event base token mint should match with actual base token mint"
        );
        assert.equal(
          event.data.pcTokenMint.toString(),
          pcMint.toString(),
          "Event pc token mint should match with actual pc token mint"
        );
        assert.equal(
          event.data.baseTokenAmount.toString(),
          baseMintAmount.toString(),
          "Event base token amount should match with actual base token amount"
        );
        assert.equal(
          event.data.pcTokenAmount.toString(),
          pcMintAmount.toString(),
          "Event pc token amount should match with actual pc token amount"
        );
      }
    }

    assert.equal(logEmitted, true, "Should emit event");
  });

  it("Initialize liquidity pool (Wsol as base mint)", async () => {
    const ammPdaIndex = new anchor.BN(2);
    const {
      ammPda,
      baseTokenVault,
      pcTokenVault,
      lpTokenMint,
      liquidityProviderLpTokenAta,
      liquidityProviderPCTokenAta,
      liquidityProviderBaseTokenAta,
      baseMint,
      pcMint,
      baseMintAmount,
      pcMintAmount,
    } = await prepareInitalizeLiquidityPool(
      2_000_000_000,
      1_000_000_000,
      new anchor.BN(2),
      true
    );
    await wrapSol(liquidityProvider, liquidityProviderBaseTokenAta);
    const lpTokenMintAmount = 414213562;
    let txSig;
    try {
      txSig = await program.methods
        .initializeLiquidity(
          lpMintDecimal,
          ammPdaIndex,
          baseMint,
          pcMint,
          baseMintAmount,
          pcMintAmount
        )
        .accounts({
          liquidityProvider: liquidityProvider.publicKey,
          ammPda: ammPda,
          baseTokenVault: baseTokenVault,
          pcTokenVault: pcTokenVault,
          lpTokenMint: lpTokenMint,
          liquidityProviderLpTokenAta: liquidityProviderLpTokenAta,
          baseTokenMint: baseMint,
          pcTokenMint: pcMint,
          liquidityProviderBaseTokenAta: liquidityProviderBaseTokenAta,
          liquidityProviderPcTokenAta: liquidityProviderPCTokenAta,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([liquidityProvider])
        .rpc({ commitment: "confirmed" });
    } catch (err) {
      console.log("Error");
      console.log(err);
    }

    const baseTokenAccount = await getAccount(
      provider.connection,
      baseTokenVault
    );
    const pcTokenAccount = await getAccount(provider.connection, pcTokenVault);
    const lpTokenAccount = await getAccount(
      provider.connection,
      liquidityProviderLpTokenAta
    );
    assert.equal(
      baseTokenAccount.amount.toString(),
      baseMintAmount.toString(),
      "Base token mint amount doesn't match"
    );

    assert.equal(
      lpTokenAccount.amount.toString(),
      lpTokenMintAmount.toString(),
      "Lp token mint amount doesn't match"
    );
    assert.equal(
      pcTokenAccount.amount.toString(),
      pcMintAmount.toString(),
      "Pc token mint amount doesn't match"
    );
    // Check emitted event

    const tx = await provider.connection.getParsedTransaction(
      txSig,
      "confirmed"
    );
    const eventParser = new anchor.EventParser(
      program.programId,
      new anchor.BorshCoder(program.idl)
    );
    const events = eventParser.parseLogs(tx.meta.logMessages);

    let logEmitted = false;
    for (let event of events) {
      if (event.name == "initializeLiquidityPoolEvent") {
        logEmitted = true;
        assert.equal(
          event.data.liquidityProvider.toString(),
          liquidityProvider.publicKey.toString(),
          "Event liquidity provder should match with actual liquidity provder"
        );
        assert.equal(
          event.data.baseTokenMint.toString(),
          baseMint.toString(),
          "Event base token mint should match with actual base token mint"
        );
        assert.equal(
          event.data.pcTokenMint.toString(),
          pcMint.toString(),
          "Event pc token mint should match with actual pc token mint"
        );
        assert.equal(
          event.data.baseTokenAmount.toString(),
          baseMintAmount.toString(),
          "Event base token amount should match with actual base token amount"
        );
        assert.equal(
          event.data.pcTokenAmount.toString(),
          pcMintAmount.toString(),
          "Event pc token amount should match with actual pc token amount"
        );
      }
    }
    assert.equal(logEmitted, true, "Should emit event");
  });
});
