import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Solcbd } from "../target/types/solcbd";
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

  it("Is initialized!", async () => {
    // Add your test here.

    let randomID = anchor.web3.Keypair.generate();
    const provider = anchor.AnchorProvider.local();

    let 
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
});
