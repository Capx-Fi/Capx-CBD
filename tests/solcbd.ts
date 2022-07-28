import * as anchor from "@project-serum/anchor";
import {
    Program
} from "@project-serum/anchor";
import {
    Solcbd
} from "../target/types/solcbd";

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

    const program = anchor.workspace.Solcbd as Program < Solcbd > ;

    let [vaultPDA, bump_vault] = [null, null];
    let [vaultPDA2, bump_vault2] = [null, null];
    let [vaultPDA3, bump_vault3] = [null, null];
    let [vaultPDA4, bump_vault4] = [null, null];
    let [vaultPDA5, bump_vault5] = [null, null];
    let [nftTarget, bump_vault6] = [null, null];
    let randomID;
    let newmint;
    let baseinit;
    let usdcmint;
    let base_ata;
    let tokenmint;
    let token_ata;

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

        [vaultPDA, bump_vault] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("project-data"),
                randomID.publicKey.toBuffer()
            ],
            program.programId
        );

        [vaultPDA3, bump_vault3] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("project-vault"),
                randomID.publicKey.toBuffer(),
                usdcmint.publicKey.toBuffer()
            ],
            program.programId
        );

        const tx = await program.methods.initializeProject(
            randomID.publicKey,
            "QmUfo11awv8Aa9ppnL4WhqJ7KMqrG4vdCWnAozrLzs9FNa",
            new anchor.BN(3000),
            new anchor.BN(4000),
            [new anchor.BN(5000)],
            new anchor.BN(2),
            [new anchor.BN(2)],
            [new anchor.BN(1659186531)],
            [new anchor.BN(9000)],
        ).accounts({
            baseAccount: baseinit.publicKey,
            projectAccount: vaultPDA,
            vaultAccount: vaultPDA3,
            usdcmint: usdcmint.publicKey,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
            user: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId
        }).rpc();
        // console.log("Your transaction signature", tx);

        let dt = await program.account.projectAccount.fetch(vaultPDA);
        await console.log(dt)

    });

    it("Initialize CBD Tag", async () => {

        // newmint = anchor.web3.Keypair.generate();

        let type = "0"; 

        [vaultPDA2, bump_vault2] = await PublicKey.findProgramAddress(
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
            projectAccount: vaultPDA,
            dataAccount: vaultPDA2,
            usdcmint: usdcmint.publicKey,
            user: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID
        }).instruction();

        let tx_data = new Transaction()
            .add(tx);

        await program.provider.sendAndConfirm(tx_data, []);

        let dt = await program.account.dataAccount.fetch(vaultPDA2);
        await console.log(dt)

        // console.log("Balance In ATA for NFT: ", await program.provider.connection.getTokenAccountBalance(def_ata));

        // console.log("Balance In ATA of USDC mint: ", await program.provider.connection.getTokenAccountBalance(base_ata));

        // console.log("Balance In PDA vault of project: ", await program.provider.connection.getTokenAccountBalance(vaultPDA3));

        let dt2 = await program.account.projectAccount.fetch(vaultPDA);
        await console.log(dt2)
    });

    it("Make NFT", async () => {

        newmint = anchor.web3.Keypair.generate();

        [nftTarget, bump_vault6] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("nft-data-target"),
                newmint.publicKey.toBuffer()
            ],
            program.programId
        );

        let def_ata = await spl.getAssociatedTokenAddress(newmint.publicKey, provider.wallet.publicKey, false, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID);
        const tx = await program.methods.mintCbd(
            randomID.publicKey,
            "0",
            bump_vault3
        ).accounts({
            baseAccount: baseinit.publicKey,
            projectAccount: vaultPDA,
            mint: newmint.publicKey,
            derAta: def_ata,
            baseAta: base_ata,
            vaultAccount: vaultPDA3,
            dataAccount: vaultPDA2,
            nftAccount: nftTarget,
            user: provider.wallet.publicKey,
            associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID
        }).instruction();

        let tx_data = new Transaction()
            .add(tx);

        await program.provider.sendAndConfirm(tx_data, [newmint]).catch(console.error);

        let dt = await program.account.dataAccount.fetch(vaultPDA2);
        await console.log(dt)

        console.log("Balance In ATA for NFT: ", await program.provider.connection.getTokenAccountBalance(def_ata));

        console.log("Balance In ATA of USDC mint: ", await program.provider.connection.getTokenAccountBalance(base_ata));

        console.log("Balance In PDA vault of project: ", await program.provider.connection.getTokenAccountBalance(vaultPDA3));

        let dt2 = await program.account.projectAccount.fetch(vaultPDA);
        await console.log(dt2)

        let dt3 = await program.account.nftAccount.fetch(nftTarget);
        await console.log("NFT PDA address - ",nftTarget.toBase58())
        await console.log("NFT PDA data - ",dt3.datatarget.toBase58())

        await console.log("Target Acconut which has data and authority - ",dt3.datatarget.toBase58())
    });


    it("Make NFT again", async () => {

        newmint = anchor.web3.Keypair.generate();

        [nftTarget, bump_vault6] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("nft-data-target"),
                newmint.publicKey.toBuffer()
            ],
            program.programId
        );

        let def_ata = await spl.getAssociatedTokenAddress(newmint.publicKey, provider.wallet.publicKey, false, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID);
        const tx = await program.methods.mintCbd(
            randomID.publicKey,
            "0",
            bump_vault3
        ).accounts({
            baseAccount: baseinit.publicKey,
            projectAccount: vaultPDA,
            mint: newmint.publicKey,
            derAta: def_ata,
            baseAta: base_ata,
            vaultAccount: vaultPDA3,
            dataAccount: vaultPDA2,
            nftAccount: nftTarget,
            user: provider.wallet.publicKey,
            associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID
        }).instruction();

        let tx_data = new Transaction()
            .add(tx);

        await program.provider.sendAndConfirm(tx_data, [newmint]).catch(console.error);

        let dt = await program.account.dataAccount.fetch(vaultPDA2);
        await console.log(dt)

        console.log("Balance In ATA for NFT: ", await program.provider.connection.getTokenAccountBalance(def_ata));

        console.log("Balance In ATA of USDC mint: ", await program.provider.connection.getTokenAccountBalance(base_ata));

        console.log("Balance In PDA vault of project: ", await program.provider.connection.getTokenAccountBalance(vaultPDA3));

        let dt2 = await program.account.projectAccount.fetch(vaultPDA);
        await console.log(dt2)

        let dt3 = await program.account.nftAccount.fetch(nftTarget);
        await console.log("NFT PDA address - ",nftTarget.toBase58())
        await console.log("NFT PDA data - ",dt3.datatarget.toBase58())

        await console.log("Target Acconut which has data and authority - ",dt3.datatarget.toBase58())
    });


    // it("Initiate Redemption", async () => {

    //     tokenmint = anchor.web3.Keypair.generate();

    //     token_ata = await spl.getAssociatedTokenAddress(tokenmint.publicKey, provider.wallet.publicKey, false, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID);

    //     let create_mint_tx = new Transaction().add(
    //             // create mint account
    //             SystemProgram.createAccount({
    //                 fromPubkey: provider.wallet.publicKey,
    //                 newAccountPubkey: tokenmint.publicKey,
    //                 space: spl.MintLayout.span,
    //                 lamports: await spl.getMinimumBalanceForRentExemptMint(program.provider.connection),
    //                 programId: spl.TOKEN_PROGRAM_ID,
    //             }),
    //             // init mint account
    //             spl.createInitializeMintInstruction(tokenmint.publicKey, 9, provider.wallet.publicKey, provider.wallet.publicKey, spl.TOKEN_PROGRAM_ID)
    //         )
    //         .add(
    //             spl.createAssociatedTokenAccountInstruction(
    //                 provider.wallet.publicKey, token_ata, provider.wallet.publicKey, tokenmint.publicKey, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID
    //             )
    //         ).add(
    //             spl.createMintToInstruction( // always TOKEN_PROGRAM_ID
    //                 tokenmint.publicKey, // mint
    //                 token_ata, // receiver (sholud be a token account)
    //                 provider.wallet.publicKey, // mint authority
    //                 7e16,
    //                 [], // only multisig account will use. leave it empty now.
    //                 spl.TOKEN_PROGRAM_ID, // amount. if your decimals is 8, you mint 10^8 for 1 token.
    //             ));

    //     await program.provider.sendAndConfirm(create_mint_tx, [tokenmint]);

    //     [vaultPDA4, bump_vault4] = await PublicKey.findProgramAddress(
    //         [
    //             anchor.utils.bytes.utf8.encode("redemption-data"),
    //             randomID.publicKey.toBuffer()
    //         ],
    //         program.programId
    //     );

    //     [vaultPDA5, bump_vault5] = await PublicKey.findProgramAddress(
    //         [
    //             anchor.utils.bytes.utf8.encode("redemption-vault"),
    //             randomID.publicKey.toBuffer(),
    //             tokenmint.publicKey.toBuffer()
    //         ],
    //         program.programId
    //     );

    //     console.log("Balance In ATA of token mint: ", await program.provider.connection.getTokenAccountBalance(token_ata));


    //     const tx = await program.methods.initializeRedemption(randomID.publicKey, new anchor.BN(1000000)).accounts({
    //         baseAccount: baseinit.publicKey,
    //         projectAccount: vaultPDA,
    //         redemptionAccount: vaultPDA4,
    //         redemptionVault: vaultPDA5,
    //         tokenAta: token_ata,
    //         projectToken: tokenmint.publicKey,
    //         usdcmint: usdcmint.publicKey,
    //         poolusdc: base_ata,
    //         pooltoken: token_ata,
    //         user: provider.wallet.publicKey,
    //         systemProgram: anchor.web3.SystemProgram.programId,
    //         tokenProgram: spl.TOKEN_PROGRAM_ID

    //     }).rpc();

    //     console.log("Balance In ATA of token mint: ", await program.provider.connection.getTokenAccountBalance(token_ata));



    // })

});