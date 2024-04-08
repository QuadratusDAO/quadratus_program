import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Quadra } from "../target/types/quadra";

import {
  createMint,
  getAssociatedTokenAddress,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Connection, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import assert from "assert";

const DECIMALS_PER_TOKEN = 1000000;

const RPC_URL = "http://127.0.0.1:8899";

// mute on devnet
// const RPC_URL = "";

const createTokenMint = async (
  connection: any,
  payer: anchor.Wallet,
  mintKeypair: anchor.web3.Keypair
) => {
  try {
    const mint = await createMint(
      connection,
      payer.payer,
      payer.publicKey,
      payer.publicKey,
      6,
      mintKeypair
    );

    // console.log(mint);
  } catch (e) {
    console.log(e);
  }
};

describe("quadra", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;

  console.log("payer: ", payer.publicKey.toBase58());
  const connection = new Connection(RPC_URL, "confirmed");
  const program = anchor.workspace.Quadra as Program<Quadra>;

  const secondPayer = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array([
      52, 248, 220, 121, 94, 96, 20, 95, 134, 125, 131, 73, 209, 240, 41, 137,
      31, 83, 23, 131, 218, 146, 132, 255, 144, 198, 98, 140, 201, 49, 163, 247,
      43, 84, 178, 209, 146, 59, 42, 75, 248, 199, 91, 149, 58, 245, 223, 25,
      72, 82, 78, 9, 19, 142, 202, 128, 251, 29, 37, 159, 215, 104, 240, 27,
    ])
  );

  // 3mQRvL823f1yQxPfNRkxqSEjqp5qWUQJtAsMK58s1gLY
  // mute on devnet
  const governanceMintKeypair = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array([
      217, 94, 193, 238, 36, 134, 194, 54, 29, 7, 206, 47, 218, 102, 202, 86,
      48, 97, 35, 242, 188, 233, 238, 136, 172, 121, 243, 138, 5, 16, 246, 38,
      41, 23, 107, 227, 81, 219, 126, 70, 37, 82, 150, 235, 79, 176, 170, 66,
      100, 29, 135, 246, 211, 239, 183, 169, 171, 112, 112, 195, 176, 163, 179,
      105,
    ])
  );

  const beneficiary = new anchor.web3.Keypair();

  const [feePDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("fee")],
    program.programId
  );

  const [daoPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("dao"), payer.publicKey.toBuffer()],
    program.programId
  );

  const [adminPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("admin"), daoPDA.toBuffer(), payer.publicKey.toBuffer()],
    program.programId
  );

  const [treasuryPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("treasury_vault"), daoPDA.toBuffer()],
    program.programId
  );

  const [burnPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("burn"), daoPDA.toBuffer()],
    program.programId
  );

  const [membershipPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("membership"),
      daoPDA.toBuffer(),
      secondPayer.publicKey.toBuffer(),
    ],
    program.programId
  );

  const [proposalPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("proposal"),
      daoPDA.toBuffer(),
      new anchor.BN(0).toBuffer("le", 8),
    ],
    program.programId
  );

  const [userProposalVotesPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("user_proposal_votes"),
      secondPayer.publicKey.toBuffer(),
      proposalPDA.toBuffer(),
    ],
    program.programId
  );

  it("airdrops SOL to the payer", async () => {
    // mute on devnet
    const lamports = LAMPORTS_PER_SOL;

    const signature = await connection.requestAirdrop(
      secondPayer.publicKey,
      lamports
    );

    await connection.confirmTransaction(signature);

    await createTokenMint(connection, payer, governanceMintKeypair);
    // end mute

    const userTokenMintAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      secondPayer.publicKey
    );

    await mintTo(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      userTokenMintAccount.address,
      payer.payer,
      1000000 * DECIMALS_PER_TOKEN
    );

    // create associated token account
    const SecondUserTokenMintAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      payer.publicKey
    );

    // mint tokens to the user
    await mintTo(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      SecondUserTokenMintAccount.address,
      payer.payer,
      1000 * DECIMALS_PER_TOKEN
    );
  });

  it("initializes the fee account", async () => {
    await program.methods
      .initializeFeeAccount()
      .accounts({
        feeAccount: feePDA,
        user: payer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc()
      .catch((e) => {
        console.log(e);
      });

    const feeAccount = await program.account.feeAccount.fetch(feePDA);

    assert.ok(feeAccount);
  });

  it("creates the DAO", async () => {
    const name = "Quadratus DAO";
    const image =
      "https://pbs.twimg.com/profile_images/1775289008731324416/-LqF3pPu_400x400.jpg";
    const min_yes_votes = new anchor.BN(1000);
    const proposal_creation_fee = new anchor.BN(1 * DECIMALS_PER_TOKEN);
    const membership_fee = new anchor.BN(100 * DECIMALS_PER_TOKEN);

    const mintAndSendTokens = async (
      mintPubKey: PublicKey,
      destination: PublicKey,
      amount: number
    ) => {
      await mintTo(
        connection,
        payer.payer,
        mintPubKey,
        destination,
        payer.payer,
        amount * DECIMALS_PER_TOKEN
      );
    };

    // mute on devnet
    await createTokenMint(connection, payer, governanceMintKeypair);

    const userTokenMintAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      payer.publicKey
    );

    // mute on devnet
    await mintAndSendTokens(
      governanceMintKeypair.publicKey,
      userTokenMintAccount.address,
      1000000
    );

    await program.methods
      .createDao(
        name,
        image,
        min_yes_votes,
        proposal_creation_fee,
        membership_fee
      )
      .accounts({
        dao: daoPDA,
        admin: adminPDA,
        treasuryVault: treasuryPDA,
        burnVault: burnPDA,
        feeAccount: feePDA,
        user: payer.publicKey,
        tokenMint: governanceMintKeypair.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc()
      .catch((e) => {
        console.log(e);
      });

    const dao = await program.account.dao.fetch(daoPDA);

    assert.ok(dao);
  });

  it("joins the DAO", async () => {
    const userTokenMintAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      secondPayer.publicKey
    );

    await program.methods
      .joinDao()
      .accounts({
        dao: daoPDA,
        membership: membershipPDA,
        treasuryVault: treasuryPDA,
        userTokenMintAccount: userTokenMintAccount.address,
        user: secondPayer.publicKey,
        tokenMint: governanceMintKeypair.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([secondPayer])
      .rpc()
      .catch((e) => {
        console.log(e);
      });

    const daoAfter = await program.account.dao.fetch(daoPDA);

    assert.ok(daoAfter);
  });

  it("creates the proposal", async () => {
    const token_amount = new anchor.BN(1 * DECIMALS_PER_TOKEN); // TODO: check for decimals
    const end_date_in_seconds = Date.now() / 1000 + 60 * 60 * 24 * 3;
    const end_date = new anchor.BN(end_date_in_seconds);
    const title = "Allocate tokens for airdrop.";
    const description = `This proposal outlines a strategic plan to allocate a portion of our DAO's token reserves towards a targeted airdrop program.\n The primary goal is to incentivize participation, attract new members, and stimulate community engagement. By distributing tokens directly to the wallets of active and potential community members, we aim to enhance the decentralized governance model of our DAO and reward those who contribute meaningfully to its growth.`;
    const action = 1; // 0 = burn, 1 = transfer
    const burnOnVote = false;

    const userTokenMintAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      payer.publicKey
    );

    const SecondUserTokenMintAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      secondPayer.publicKey
    );

    const beneficiary_ata = await getAssociatedTokenAddress(
      governanceMintKeypair.publicKey,
      beneficiary.publicKey
    );

    await program.methods
      .createProposal(
        token_amount,
        end_date,
        title,
        description,
        action,
        burnOnVote
      )
      .accounts({
        dao: daoPDA,
        proposal: proposalPDA,
        treasuryVault: treasuryPDA,
        beneficiary: beneficiary_ata,
        beneficiaryOwner: beneficiary.publicKey,
        membership: membershipPDA,
        user: secondPayer.publicKey,
        tokenMint: governanceMintKeypair.publicKey,
        userTokenMintAccount: SecondUserTokenMintAccount.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([secondPayer])
      .rpc()
      .catch((e) => {
        console.log(e);
      });

    const proposal = await program.account.proposal.fetch(proposalPDA);

    assert.ok(proposal);
  });

  it("votes on the proposal", async () => {
    // create associated token account
    const userTokenMintAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      payer.publicKey
    );

    const SecondUserTokenMintAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      secondPayer.publicKey
    );

    // mint tokens to the user
    await mintTo(
      connection,
      payer.payer,
      governanceMintKeypair.publicKey,
      SecondUserTokenMintAccount.address,
      payer.payer,
      1000 * DECIMALS_PER_TOKEN
    );

    const amount = new anchor.BN(100); // amount of votes
    const side = 1; // 1 for yes, 0 for no

    await program.methods
      .voteOnProposal(amount, side)
      .accounts({
        dao: daoPDA,
        proposal: proposalPDA,
        userProposalVotes: userProposalVotesPDA,
        treasuryVault: treasuryPDA,
        burnVault: burnPDA,
        userTokenMintAccount: SecondUserTokenMintAccount.address,
        membership: membershipPDA,
        user: secondPayer.publicKey,
        tokenMint: governanceMintKeypair.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([secondPayer])
      .rpc()
      .catch((e) => {
        console.log(e);
      });
  });

  it("executes a completed proposal", async () => {
    // wait 1 minutes
    // await new Promise((resolve) => setTimeout(resolve, 60000));

    const beneficiary_ata = await getAssociatedTokenAddress(
      governanceMintKeypair.publicKey,
      beneficiary.publicKey
    );

    await program.methods
      .executeProposal()
      .accounts({
        dao: daoPDA,
        proposal: proposalPDA,
        treasuryVault: treasuryPDA,
        burnVault: burnPDA,
        beneficiary: beneficiary_ata,
        beneficiaryOwner: beneficiary.publicKey,
        user: payer.publicKey,
        tokenMint: governanceMintKeypair.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc()
      .catch((e) => {
        console.log(e);
      });

    const proposal = await program.account.proposal.fetch(proposalPDA);

    // const beneficiaryBalance = await connection.getTokenAccountBalance(
    //   beneficiary_ata
    // );

    // console.log(beneficiaryBalance);
    // console.log(beneficiary_ata);

    assert.ok(proposal.executed);
  });
});
