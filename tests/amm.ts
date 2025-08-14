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
  const user = anchor.web3.Keypair.generate();
  const depositor = anchor.web3.Keypair.generate();
  const connection = provider.connection;
  const lpMintDecimal: number = 9;

  let ammVariables = {};
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
    userKeypair: anchor.web3.Keypair,
    isNativeBase: boolean = false,
    isNativePc: boolean = false,
    baseMintAddress: any = null,
    pcMintAddress: any = null
  ) {
    let baseMint = baseMintAddress;
    let pcMint = pcMintAddress;
    if (isNativeBase && !baseMint && !pcMint) {
      baseMint = NATIVE_MINT;
      ({ pcMint } = await createBaseAndPCMint());
    } else if (isNativePc && !baseMint && !pcMint) {
      pcMint = NATIVE_MINT;
      ({ baseMint } = await createBaseAndPCMint());
    } else if (!baseMint && !pcMint) {
      ({ baseMint, pcMint } = await createBaseAndPCMint());
    }

    const userBaseTokenAta = await getAssociatedTokenAddress(
      baseMint,
      userKeypair.publicKey
    );
    const userPCTokenAta = await getAssociatedTokenAddress(
      pcMint,
      userKeypair.publicKey
    );

    if (!isNativeBase) {
      await createAssociatedTokenAccount(
        connection,
        provider.wallet.payer,
        baseMint,
        userKeypair.publicKey
      );
      await mintTo(
        connection,
        provider.wallet.payer,
        baseMint,
        userBaseTokenAta,
        provider.wallet.payer,
        baseMintToAmount
      );
    }
    if (!isNativePc) {
      await createAssociatedTokenAccount(
        connection,
        provider.wallet.payer,
        pcMint,
        userKeypair.publicKey
      );
      await mintTo(
        connection,
        provider.wallet.payer,
        pcMint,
        userPCTokenAta,
        provider.wallet.payer,
        pcMintToAmount
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
      userKeypair.publicKey.toBuffer(),
      ammPda.toBuffer(),
    ]);
    ammVariables[ammPdaIndex.toNumber()] = {
      ammPda,
      baseTokenVault,
      pcTokenVault,
      lpTokenMint,
      liquidityProviderLpTokenAta,
      userPCTokenAta,
      userBaseTokenAta,
      baseMint,
      pcMint,
      baseMintAmount,
      pcMintAmount,
    };
    return ammVariables[ammPdaIndex.toNumber()];
  }
  async function prepareSwap(
    baseMintToAmount: number,
    pcMintToAmount: number,
    ammPdaIndex: number,
    userKeypair: anchor.web3.Keypair,
    isNativeBase: boolean = false,
    isNativePc: boolean = false
  ) {
    const { baseMint, pcMint } = ammVariables[ammPdaIndex];
    const userBaseTokenAta = await getAssociatedTokenAddress(
      baseMint,
      userKeypair.publicKey
    );
    const userPCTokenAta = await getAssociatedTokenAddress(
      pcMint,
      userKeypair.publicKey
    );

    if (!isNativeBase) {
      await createAssociatedTokenAccount(
        connection,
        provider.wallet.payer,
        baseMint,
        userKeypair.publicKey
      );
      await mintTo(
        connection,
        provider.wallet.payer,
        baseMint,
        userBaseTokenAta,
        provider.wallet.payer,
        baseMintToAmount
      );
    }
    if (!isNativePc) {
      await createAssociatedTokenAccount(
        connection,
        provider.wallet.payer,
        pcMint,
        userKeypair.publicKey
      );
      await mintTo(
        connection,
        provider.wallet.payer,
        pcMint,
        userPCTokenAta,
        provider.wallet.payer,
        pcMintToAmount
      );
    }
    return {
      userBaseTokenAta,
      userPCTokenAta,
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

  async function exchangeBaseToPc(
    inputBaseAmount: anchor.BN,
    totalPoolBaseAmount: anchor.BN,
    totalPoolPcAmount: anchor.BN
  ) {
    const maxPcAmount = inputBaseAmount
      .mul(totalPoolPcAmount)
      .div(totalPoolBaseAmount);

    return maxPcAmount;
  }

  async function exchangePcToBase(
    inputPcAmount: anchor.BN,
    totalPoolBaseAmount: anchor.BN,
    totalPoolPcAmount: anchor.BN
  ) {
    const maxBaseAmount = inputPcAmount
      .mul(totalPoolBaseAmount)
      .div(totalPoolPcAmount);

    return maxBaseAmount;
  }
  before(async () => {
    const airdropSig = await provider.connection.requestAirdrop(
      liquidityProvider.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 100
    );
    await provider.connection.confirmTransaction(airdropSig);
    const airdropSigUser = await provider.connection.requestAirdrop(
      user.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 100
    );
    await provider.connection.confirmTransaction(airdropSigUser);
    const airdropSigDepositor = await provider.connection.requestAirdrop(
      depositor.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 100
    );
    await provider.connection.confirmTransaction(airdropSigDepositor);
  });
  it("Initialize liquidity pool", async () => {
    const ammPdaIndex = new anchor.BN(1);
    const {
      ammPda,
      baseTokenVault,
      pcTokenVault,
      lpTokenMint,
      liquidityProviderLpTokenAta,
      userPCTokenAta: liquidityProviderPCTokenAta,
      userBaseTokenAta: liquidityProviderBaseTokenAta,
      baseMint,
      pcMint,
      baseMintAmount,
      pcMintAmount,
    } = await prepareInitalizeLiquidityPool(
      2_000_000_000,
      1_000_000_000,
      ammPdaIndex,
      liquidityProvider
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
      userPCTokenAta: liquidityProviderPCTokenAta,
      userBaseTokenAta: liquidityProviderBaseTokenAta,
      baseMint,
      pcMint,
      baseMintAmount,
      pcMintAmount,
    } = await prepareInitalizeLiquidityPool(
      2_000_000_000,
      1_000_000_000,
      ammPdaIndex,
      liquidityProvider,
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

  it("Swap Coin2Pc", async () => {
    const ammPdaIndex = new anchor.BN(2);
    const {
      ammPda,
      baseTokenVault,
      pcTokenVault,

      baseMint,
      pcMint,
    } = ammVariables[ammPdaIndex.toNumber()];
    const { userBaseTokenAta, userPCTokenAta } = await prepareSwap(
      2_000_000_000,
      1_000_000_000,
      ammPdaIndex.toNumber(),
      user,
      true
    );
    await wrapSol(user, userBaseTokenAta);
    const amountIn = new anchor.BN(2_000_000_00);
    await program.methods
      .swapBaseIn(ammPdaIndex, amountIn, amountIn)
      .accounts({
        user: user.publicKey,
        ammPda: ammPda,
        baseTokenVault: baseTokenVault,
        pcTokenVault: pcTokenVault,
        userSourceAta: userBaseTokenAta,
        userDestinationAta: userPCTokenAta,
        baseTokenMint: baseMint,
        pcTokenMint: pcMint,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const userbaseTokenAccount = await getAccount(
      provider.connection,
      userBaseTokenAta
    );
    const userpcTokenAccount = await getAccount(
      provider.connection,
      userPCTokenAta
    );

    console.log("Base:  ", userbaseTokenAccount.amount.toString());
    console.log("PC:  ", userpcTokenAccount.amount.toString());
  });

  it("Despoit into pool", async () => {
    const ammPdaIndex = new anchor.BN(2);

    const {
      ammPda,
      baseTokenVault,
      pcTokenVault,

      baseMint,
      pcMint,
    } = ammVariables[ammPdaIndex.toNumber()];
    const {
      lpTokenMint,
      liquidityProviderLpTokenAta,
      userPCTokenAta: liquidityProviderPCTokenAta,
      userBaseTokenAta: liquidityProviderBaseTokenAta,

      baseMintAmount,
      pcMintAmount,
    } = await prepareInitalizeLiquidityPool(
      2_000_000_000,
      1_000_000_000,
      ammPdaIndex,
      depositor,
      true,
      false,
      baseMint,
      pcMint
    );

    await wrapSol(depositor, liquidityProviderBaseTokenAta);

    const userBaseInput = new anchor.BN(2_000_000_00);

    let baseTokenVaultAccount = await getAccount(
      provider.connection,
      baseTokenVault
    );
    let pcTokenvaultAccount = await getAccount(
      provider.connection,
      pcTokenVault
    );

    const baseTokenVaultAmount = new anchor.BN(baseTokenVaultAccount.amount);
    const pcTokenVaultAmount = new anchor.BN(pcTokenvaultAccount.amount);

    const maxPcAmount = await exchangeBaseToPc(
      userBaseInput,
      baseTokenVaultAmount,
      pcTokenVaultAmount
    );
    const base_side: number = 0;

    const baseAta = await getAccount(
      provider.connection,
      liquidityProviderBaseTokenAta
    );
    const pcAta = await getAccount(
      provider.connection,
      liquidityProviderPCTokenAta
    );

    await program.methods
      .deposit(
        lpMintDecimal,
        ammPdaIndex,
        baseMint,
        pcMint,
        userBaseInput,
        maxPcAmount,
        base_side
      )
      .accounts({
        user: depositor.publicKey,
        ammPda: ammPda,
        baseTokenVault: baseTokenVault,
        pcTokenVault: pcTokenVault,
        lpTokenMint: lpTokenMint,
        baseTokenMint: baseMint,
        pcTokenMint: pcMint,
        liquidityProviderLpTokenAta: liquidityProviderLpTokenAta,
        liquidityProviderBaseTokenAta: liquidityProviderBaseTokenAta,
        liquidityProviderPcTokenAta: liquidityProviderPCTokenAta,
      })
      .signers([depositor])
      .rpc();
    let lpAta = await getAccount(
      provider.connection,
      liquidityProviderLpTokenAta
    );
    console.log(lpAta.amount.toString());
    baseTokenVaultAccount = await getAccount(
      provider.connection,
      baseTokenVault
    );

    pcTokenvaultAccount = await getAccount(provider.connection, pcTokenVault);
    const finalBaseTokenCountInVault = userBaseInput.add(baseTokenVaultAmount);
    const finalPcTokenCountInVault = maxPcAmount.add(pcTokenVaultAmount);

    assert.equal(
      finalBaseTokenCountInVault.toString(),
      baseTokenVaultAccount.amount.toString()
    );
    assert.equal(
      finalPcTokenCountInVault.toString(),
      pcTokenvaultAccount.amount.toString()
    );
  });
});
