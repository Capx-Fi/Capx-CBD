import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Solcbd } from "../target/types/solcbd";

import * as spl from "@solana/spl-token";

const {
  Connection,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
  PublicKey,
  SystemProgram,
} = anchor.web3;

describe("solcbd", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Solcbd as Program<Solcbd>;

  let [vaultPDA, bump_vault] = [null,null];
  let [vaultPDA2, bump_vault2] = [null,null];
  let randomID;
  let newmint;
  
  const provider = anchor.AnchorProvider.local();

  it("Is initialized!", async () => {
    // Add your test here.

    randomID = anchor.web3.Keypair.generate();

    [vaultPDA, bump_vault] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("project-data"),
        randomID.publicKey.toBuffer()
      ],
      program.programId
    );

    const tx = await program.methods.initializeProject(
      randomID.publicKey,
      "QmUfo11awv8Aa9ppnL4WhqJ7KMqrG4vdCWnAozrLzs9FNa",
      new anchor.BN(3000),
      new anchor.BN(4000),
      new anchor.BN(5000),
      new anchor.BN(7000),
      [new anchor.BN(7000)],
      [new anchor.BN(1659186531)],
      [new anchor.BN(9000)],
    ).accounts({
      projectAccount: vaultPDA,
      user: provider.wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId
    }).rpc();
    // console.log("Your transaction signature", tx);

    let dt = await program.account.projectAccount.fetch(vaultPDA);
      await console.log(dt)
      
  });

  it("Make NFT", async()=>{

    newmint = anchor.web3.Keypair.generate();

    [vaultPDA2, bump_vault2] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("nft-data"),
        randomID.publicKey.toBuffer(),
        newmint.publicKey.toBuffer()
      ],
      program.programId
    );

    let def_ata = await spl.getAssociatedTokenAddress(newmint.publicKey, provider.wallet.publicKey, false, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID);
    const tx = await program.methods.mintCbd(
      randomID.publicKey,
      new anchor.BN(0)
    ).accounts({
      projectAccount: vaultPDA,
      mint: newmint.publicKey,
      derAta : def_ata,
      dataAccount : vaultPDA2,
      user: provider.wallet.publicKey,
      associatedTokenProgram : spl.ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID
    }).instruction();

    let tx_data = new Transaction()
    .add(tx);

    await program.provider.sendAndConfirm(tx_data, [newmint]);

    let dt = await program.account.dataAccount.fetch(vaultPDA2);
    await console.log(dt)

    console.log("Balance In ATA: ", await program.provider.connection.getTokenAccountBalance(def_ata));

  });

});
