import * as anchor from "@project-serum/anchor";
import {
    Program
} from "@project-serum/anchor";
import {
    Solcbd
} from "../target/types/solcbd";
import { findMetadataPda } from "@metaplex-foundation/js";

import * as spl from "@solana/spl-token";
import { getMint } from "@solana/spl-token";

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

    const program = anchor.workspace.Solcbd as Program < Solcbd > ;

    let [projectAccountPDA, projectAccountPDA_bump] = [null, null];
    let [projectMetaAccountPDA, projectMetaAccountPDA_bump] = [null, null];
    let [dataAccountPDA, dataAccountPDA_bump] = [null, null];
    let [vaultAccountPDA, vaultAccountPDA_bump] = [null, null];
    let [redemptionAccountPDA, redemptionAccountPDA_bump] = [null, null];
    let [redemptionVaultPDA, redemptionVaultPDA_bump] = [null, null];
    let [whitelistPDA, whitelistPDA_bump] = [null, null];
    let [nftTarget, nftTarget_bump] = [null, null];
    let randomID;
    let newmint;
    let newmint2;
    let baseinit;
    let usdcmint;
    let base_ata;
    let tokenmint;
    let token_ata;
    let def_ata;
    const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
        "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
      );

    const provider = anchor.AnchorProvider.local();

    it("initialize the program", async () => {
        // Add your test here.

        baseinit = anchor.web3.Keypair.generate();
        usdcmint = anchor.web3.Keypair.generate();

        base_ata = await spl.getAssociatedTokenAddress(usdcmint.publicKey, provider.wallet.publicKey, false, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID);

        let create_mint_tx = new Transaction().add(
                // create mint account
                SystemProgram.createAccount({
                    fromPubkey: provider.wallet.publicKey,
                    newAccountPubkey: usdcmint.publicKey,
                    space: spl.MintLayout.span,
                    lamports: await spl.getMinimumBalanceForRentExemptMint(program.provider.connection),
                    programId: spl.TOKEN_PROGRAM_ID,
                }),
                // init mint account
                spl.createInitializeMintInstruction(usdcmint.publicKey, 6, provider.wallet.publicKey, provider.wallet.publicKey, spl.TOKEN_PROGRAM_ID)
            )
            .add(
                spl.createAssociatedTokenAccountInstruction(
                    provider.wallet.publicKey, base_ata, provider.wallet.publicKey, usdcmint.publicKey, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID
                )
            ).add(
                spl.createMintToInstruction( // always TOKEN_PROGRAM_ID
                    usdcmint.publicKey, // mint
                    base_ata, // receiver (sholud be a token account)
                    provider.wallet.publicKey, // mint authority
                    3e16,
                    [], // only multisig account will use. leave it empty now.
                    spl.TOKEN_PROGRAM_ID, // amount. if your decimals is 8, you mint 10^8 for 1 token.
                ));


        const tx = await program.methods.initialize(usdcmint.publicKey).accounts({
            baseAccount: baseinit.publicKey,
            user: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId
        }).instruction();

        create_mint_tx.add(tx);

        await program.provider.sendAndConfirm(create_mint_tx, [usdcmint, baseinit]);
        // // console.log("Your transaction signature", tx);

        let dt = await program.account.initAccount.fetch(baseinit.publicKey);
        await console.log(dt)

        console.log("Balance In ATA: ", await program.provider.connection.getTokenAccountBalance(base_ata));

    });

    it("initialize Project", async () => {
        // Add your test here.

        randomID = anchor.web3.Keypair.generate();

        [projectAccountPDA, projectAccountPDA_bump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("project-data"),
                randomID.publicKey.toBuffer()
            ],
            program.programId
        );
        [projectMetaAccountPDA, projectMetaAccountPDA_bump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("project-metadata"),
                randomID.publicKey.toBuffer()
            ],
            program.programId
        );

        [vaultAccountPDA, vaultAccountPDA_bump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("project-vault"),
                randomID.publicKey.toBuffer(),
                usdcmint.publicKey.toBuffer()
            ],
            program.programId
        );

        const tx = await program.methods.initializeProject(
            randomID.publicKey,
            "CTRL - Program Controlled Token",
            "CTRL",
            ["https://bernieblume.github.io/Ctrl/Ctrl.json"],
            new anchor.BN(3000),
            new anchor.BN(4000),
            [new anchor.BN(5000)],
            new anchor.BN(2),
            [new anchor.BN(2)],
            [new anchor.BN(1690849383)],
            [new anchor.BN(900000)],
        ).accounts({
            baseAccount: baseinit.publicKey,
            projectAccount: projectAccountPDA,
            projectmetaAccount : projectMetaAccountPDA,
            vaultAccount: vaultAccountPDA,
            usdcmint: usdcmint.publicKey,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
            user: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId
        }).rpc();
        // console.log("Your transaction signature", tx);

        let dt = await program.account.projectAccount.fetch(projectAccountPDA);
        await console.log(dt)

        let dt2 = await program.account.projectMetaAccount.fetch(projectMetaAccountPDA);
        await console.log(dt2)

    });

    it("Whitelist Address", async() => {

        [whitelistPDA,whitelistPDA_bump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("project-whitelist"),
                randomID.publicKey.toBuffer(),
                provider.wallet.publicKey.toBuffer()
            ],
            program.programId
        );

        const tx = await program.methods.whitelist(
            randomID.publicKey,
            provider.wallet.publicKey
        ).accounts({
            projectAccount: projectAccountPDA,
            whiteAccount : whitelistPDA,
            user: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId
        }).rpc();

        let dt = await program.account.whiteAccount.fetch(whitelistPDA);
        await console.log(dt);

    });

    it("Initialize CBD Tag", async () => {

        // newmint = anchor.web3.Keypair.generate();

        let type = "0"; 

        [dataAccountPDA, dataAccountPDA_bump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("nft-data"),
                randomID.publicKey.toBuffer(),
                anchor.utils.bytes.utf8.encode(type)
            ],
            program.programId
        );

        const tx = await program.methods.initializeCbd(
            randomID.publicKey,
            type
        ).accounts({
            baseAccount: baseinit.publicKey,
            projectAccount: projectAccountPDA,
            dataAccount: dataAccountPDA,
            usdcmint: usdcmint.publicKey,
            user: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID
        }).instruction();

        let tx_data = new Transaction()
            .add(tx);

        await program.provider.sendAndConfirm(tx_data, []);

        let dt = await program.account.dataAccount.fetch(dataAccountPDA);
        await console.log(dt)

        // console.log("Balance In ATA for NFT: ", await program.provider.connection.getTokenAccountBalance(def_ata));

        // console.log("Balance In ATA of USDC mint: ", await program.provider.connection.getTokenAccountBalance(base_ata));

        // console.log("Balance In PDA vault of project: ", await program.provider.connection.getTokenAccountBalance(vaultAccountPDA));

        let dt2 = await program.account.projectAccount.fetch(projectAccountPDA);
        await console.log(dt2)
    });

    it("Make NFT", async () => {

        newmint = anchor.web3.Keypair.generate();

        [nftTarget, nftTarget_bump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("nft-data-target"),
                newmint.publicKey.toBuffer()
            ],
            program.programId
        );

        const metadataAddress = await findMetadataPda(newmint.publicKey);

        

        def_ata = await spl.getAssociatedTokenAddress(newmint.publicKey, provider.wallet.publicKey, false, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID);
        const tx = await program.methods.mintCbd(
            randomID.publicKey,
            "0",
            vaultAccountPDA_bump
        ).accounts({
            baseAccount: baseinit.publicKey,
            projectAccount: projectAccountPDA,
            projectmetaAccount : projectMetaAccountPDA,
            mint: newmint.publicKey,
            derAta: def_ata,
            baseAta: base_ata,
            vaultAccount: vaultAccountPDA,
            dataAccount: dataAccountPDA,
            whiteAccount: whitelistPDA,
            nftAccount: nftTarget,
            user: provider.wallet.publicKey,
            metadata : metadataAddress,
            tokenMetadataProgram : TOKEN_METADATA_PROGRAM_ID,
            associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID
        }).instruction();

        let tx_data = new Transaction()
            .add(tx);

        await program.provider.sendAndConfirm(tx_data, [newmint]).catch(console.error);

        let dt = await program.account.dataAccount.fetch(dataAccountPDA);
        await console.log(dt)

        console.log("Balance In ATA for NFT: ", await program.provider.connection.getTokenAccountBalance(def_ata));

        console.log("Balance In ATA of USDC mint: ", await program.provider.connection.getTokenAccountBalance(base_ata));

        console.log("Balance In PDA vault of project: ", await program.provider.connection.getTokenAccountBalance(vaultAccountPDA));

        let dt2 = await program.account.projectAccount.fetch(projectAccountPDA);
        await console.log(dt2)

        let dt3 = await program.account.nftAccount.fetch(nftTarget);
        await console.log("NFT PDA address - ",nftTarget.toBase58())
        await console.log("NFT PDA data - ",dt3.datatarget.toBase58())

        await console.log("Target Acconut which has data and authority - ",dt3.datatarget.toBase58())
    
        let dt4 = await program.account.whiteAccount.fetch(whitelistPDA);
        await console.log(dt4);
        
        let mintInfo = await getMint(program.provider.connection,newmint.publicKey);
        await console.log(mintInfo);
        


    });


    it("Make NFT again", async () => {

        newmint2 = anchor.web3.Keypair.generate();

        [nftTarget, nftTarget_bump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("nft-data-target"),
                newmint2.publicKey.toBuffer()
            ],
            program.programId
        );

        const metadataAddress = (await anchor.web3.PublicKey.findProgramAddress(
            [
              Buffer.from("metadata"),
              TOKEN_METADATA_PROGRAM_ID.toBuffer(),
              newmint2.publicKey.toBuffer(),
            ],
            TOKEN_METADATA_PROGRAM_ID
          ))[0];

        let def_ata = await spl.getAssociatedTokenAddress(newmint2.publicKey, provider.wallet.publicKey, false, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID);
        const tx = await program.methods.mintCbdCreator(
            randomID.publicKey,
            "0",
            provider.wallet.publicKey
        ).accounts({
            baseAccount: baseinit.publicKey,
            projectAccount: projectAccountPDA,
            projectmetaAccount : projectMetaAccountPDA,
            mint: newmint2.publicKey,
            derAta: def_ata,
            targetUser : provider.wallet.publicKey,
            dataAccount: dataAccountPDA,
            whiteAccount: whitelistPDA,
            nftAccount: nftTarget,
            user: provider.wallet.publicKey,
            metadata : metadataAddress,
            tokenMetadataProgram : TOKEN_METADATA_PROGRAM_ID,
            associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID
        }).instruction();

        let tx_data = new Transaction()
            .add(tx);

        await program.provider.sendAndConfirm(tx_data, [newmint2]).catch(console.error);

        let dt = await program.account.dataAccount.fetch(dataAccountPDA);
        await console.log(dt)

        console.log("Balance In ATA for NFT: ", await program.provider.connection.getTokenAccountBalance(def_ata));

        console.log("Balance In ATA of USDC mint: ", await program.provider.connection.getTokenAccountBalance(base_ata));

        console.log("Balance In PDA vault of project: ", await program.provider.connection.getTokenAccountBalance(vaultAccountPDA));

        let dt2 = await program.account.projectAccount.fetch(projectAccountPDA);
        await console.log(dt2)

        let dt3 = await program.account.nftAccount.fetch(nftTarget);
        await console.log("NFT PDA address - ",nftTarget.toBase58())
        await console.log("NFT PDA data - ",dt3.datatarget.toBase58())

        await console.log("Target Acconut which has data and authority - ",dt3.datatarget.toBase58())
    
        let dt4 = await program.account.whiteAccount.fetch(whitelistPDA);
        await console.log(dt4);
    });


    it("Initiate Redemption", async () => {

        tokenmint = anchor.web3.Keypair.generate();

        token_ata = await spl.getAssociatedTokenAddress(tokenmint.publicKey, provider.wallet.publicKey, false, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID);

        let create_mint_tx = new Transaction().add(
                // create mint account
                SystemProgram.createAccount({
                    fromPubkey: provider.wallet.publicKey,
                    newAccountPubkey: tokenmint.publicKey,
                    space: spl.MintLayout.span,
                    lamports: await spl.getMinimumBalanceForRentExemptMint(program.provider.connection),
                    programId: spl.TOKEN_PROGRAM_ID,
                }),
                // init mint account
                spl.createInitializeMintInstruction(tokenmint.publicKey, 9, provider.wallet.publicKey, provider.wallet.publicKey, spl.TOKEN_PROGRAM_ID)
            )
            .add(
                spl.createAssociatedTokenAccountInstruction(
                    provider.wallet.publicKey, token_ata, provider.wallet.publicKey, tokenmint.publicKey, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID
                )
            ).add(
                spl.createMintToInstruction( // always TOKEN_PROGRAM_ID
                    tokenmint.publicKey, // mint
                    token_ata, // receiver (sholud be a token account)
                    provider.wallet.publicKey, // mint authority
                    7e16,
                    [], // only multisig account will use. leave it empty now.
                    spl.TOKEN_PROGRAM_ID, // amount. if your decimals is 8, you mint 10^8 for 1 token.
                ));

        await program.provider.sendAndConfirm(create_mint_tx, [tokenmint]);

        [redemptionAccountPDA, redemptionAccountPDA_bump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("redemption-data"),
                randomID.publicKey.toBuffer()
            ],
            program.programId
        );

        

        console.log("Balance In ATA of token mint: ", await program.provider.connection.getTokenAccountBalance(token_ata));
        await console.log("Token ATA - ",token_ata.toBase58())


        const tx = await program.methods.initializeRedemption(randomID.publicKey).accounts({
            baseAccount: baseinit.publicKey,
            projectAccount: projectAccountPDA,
            redemptionAccount: redemptionAccountPDA,
            // redemptionVault: redemptionVaultPDA,
            // tokenAta: token_ata,
            projectToken: tokenmint.publicKey,
            poolusdc: base_ata,
            pooltoken: token_ata,
            user: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID

        }).rpc();

        let dt2 = await program.account.redemptionAccount.fetch(redemptionAccountPDA);
        await console.log("Creator - ",dt2.creator.toBase58())
        await console.log("ID - ",dt2.id.toBase58())
        await console.log("Token - ",dt2.token.toBase58())
        await console.log("PoolUSDC - ",dt2.poolusdc.toBase58())
        await console.log("PoolToken - ",dt2.pooltoken.toBase58())


    })


    it("Fund a redemption type", async () => {

        console.log("Balance In ATA before Funding: ", await program.provider.connection.getTokenAccountBalance(token_ata));

        let typeToFund = "0";

        [redemptionVaultPDA, redemptionVaultPDA_bump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("redemption-vault"),
                randomID.publicKey.toBuffer(),
                anchor.utils.bytes.utf8.encode(typeToFund)
            ],
            program.programId
        );

        const tx = await program.methods.fundVault(randomID.publicKey,typeToFund,new anchor.BN(300000000000000)).accounts({
            baseAccount: baseinit.publicKey,
            projectAccount : projectAccountPDA,
            redemptionAccount : redemptionAccountPDA,
            redemptionVault : redemptionVaultPDA,
            projectToken : tokenmint.publicKey,
            tokenAta : token_ata,
            user : provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID
        }).rpc();

        console.log("Balance In ATA before Funding: ", await program.provider.connection.getTokenAccountBalance(token_ata));
        
        console.log("Balance Of redemption vault: ", await program.provider.connection.getTokenAccountBalance(redemptionVaultPDA));

    });


    it("Redeem CBD", async () => {


        let typeToFund = "0";

        console.log("Balance In ATA for NFT before redeeam: ", await program.provider.connection.getTokenAccountBalance(def_ata));

        await console.log("Mint Key - ",newmint.publicKey.toBase58());
        await console.log("Def_ata - ", def_ata.toBase58());

        [nftTarget, nftTarget_bump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("nft-data-target"),
                newmint.publicKey.toBuffer()
            ],
            program.programId
        );

        console.log("Balance of token ATA before redeem: ", await program.provider.connection.getTokenAccountBalance(token_ata));
        
        const tx = await program.methods.redeemCbd(randomID.publicKey,typeToFund, redemptionVaultPDA_bump).accounts({
            mint : newmint.publicKey,
            derAta : def_ata,
            baseAccount: baseinit.publicKey,
            projectAccount : projectAccountPDA,
            dataAccount: dataAccountPDA,
            nftAccount: nftTarget,
            redemptionAccount: redemptionAccountPDA,
            redemptionVault: redemptionVaultPDA,
            tokenAta: token_ata,
            poolusdc: base_ata,
            pooltoken: token_ata,
            user: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID
            
        }).instruction();
        
        const transaction = new Transaction().add(
            spl.createApproveInstruction(def_ata,dataAccountPDA,provider.wallet.publicKey,BigInt(1),[],spl.TOKEN_PROGRAM_ID)
            ).add(tx);
            
            await program.provider.sendAndConfirm(transaction, []);
            
            console.log("Balance In ATA for NFT after redeeam: ", await program.provider.connection.getTokenAccountBalance(def_ata));
            
            console.log("Balance of token ATA after redeem: ", await program.provider.connection.getTokenAccountBalance(token_ata));
    });
        
    
    it("Withdraw Project Funds", async () => {

        console.log("Balance of USDC ATA before withdraw: ", await program.provider.connection.getTokenAccountBalance(base_ata));
        
        console.log("Balance of USDC Vault before withdraw: ", await program.provider.connection.getTokenAccountBalance(vaultAccountPDA));
        
        const tx = await program.methods.withdrawFund(randomID.publicKey,vaultAccountPDA_bump,new anchor.BN(4427)).accounts({
            projectAccount: projectAccountPDA,
            baseAccount : baseinit.publicKey,
            vaultAccount : vaultAccountPDA,
            baseAta : base_ata,
            user: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID
        }).rpc();
        
        
        console.log("Balance of USDC ATA after withdraw: ", await program.provider.connection.getTokenAccountBalance(base_ata));
        
        console.log("Balance of USDC Vault after withdraw: ", await program.provider.connection.getTokenAccountBalance(vaultAccountPDA));
    });


    it("Edit Unlock Time for TAG", async () => {

        let typeToFund = "0";

        let dt = await program.account.dataAccount.fetch(dataAccountPDA);
        await console.log(dt);
        await console.log(dt.unlocktime.toString(10));

        const tx = await program.methods.editReleaseTime(randomID.publicKey,typeToFund,new anchor.BN(1698820339)).accounts({
            projectAccount: projectAccountPDA,
            dataAccount:dataAccountPDA,
            user: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        }).rpc();

        dt = await program.account.dataAccount.fetch(dataAccountPDA);
        await console.log(dt);
        await console.log(dt.unlocktime.toString(10));

    });


});