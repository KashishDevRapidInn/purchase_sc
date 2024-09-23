import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { expect } from "chai";
import bs58 from "bs58";
import { Purchase } from "../target/types/purchase";
import dotenv from "dotenv";

dotenv.config();
const phantomSecretKeyBase58_buyer = process.env.PHANTOM_SECRET_KEY_BUYER;
const user_pk_buyer = bs58.decode(phantomSecretKeyBase58_buyer);
const phantomKeypair_buyer = Keypair.fromSecretKey(user_pk_buyer);

const phantomSecretKeyBase58_seller = process.env.PHANTOM_SECRET_KEY_SELLER;
const user_pk_seller = bs58.decode(phantomSecretKeyBase58_seller);
const phantomKeypair_seller = Keypair.fromSecretKey(user_pk_seller);

describe("purchase", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Purchase as Program<Purchase>;
  const purchaseAgreementKeypair = Keypair.generate();
  const startTime = Math.floor(Date.now() / 1000); // Current time in seconds
  const endTime = startTime + 3600; // 1 hour later
  const price = new BN(10);
  const nft_id = "6nzTzVZAVq4E5iuPMhcir8ESF9dLjCDgRV7yRgkU8Vbz";
  const itemName = "Mug";

  it("Setup Purchase!", async () => {
    console.log(anchor.web3.SystemProgram.programId);
    console.log(
      "Seller Public Key:",
      phantomKeypair_seller.publicKey.toString()
    );
    await program.methods
      .initializePurchase(price, nft_id, new BN(startTime), new BN(endTime))
      .accounts({
        purchaseAgreement: purchaseAgreementKeypair.publicKey,
        seller: phantomKeypair_seller.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([purchaseAgreementKeypair, phantomKeypair_seller])
      .rpc();

    const purchaseState = await program.account.purchaseAgreement.fetch(
      purchaseAgreementKeypair.publicKey
    );

    expect(purchaseState.seller.toString()).to.equal(
      phantomKeypair_seller.publicKey.toString()
    );
    expect(purchaseState.price.toNumber()).to.equal(price.toNumber());
    expect(purchaseState.status).to.eql({ itemNotTransferred: {} });
    expect(purchaseState.itemName).to.equal(itemName);
  });
  it("Making Payment", async () => {
    const system_program = anchor.web3.SystemProgram.programId;
    console.log(system_program);
    await program.methods
      .makePayment()
      .accounts({
        purchaseAgreement: purchaseAgreementKeypair.publicKey,
        buyer: phantomKeypair_buyer.publicKey,
        seller: phantomKeypair_seller.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([phantomKeypair_buyer, phantomKeypair_seller])
      .rpc();
    const purchaseState = await program.account.purchaseAgreement.fetch(
      purchaseAgreementKeypair.publicKey
    );
    expect(purchaseState.status).to.eql({ paymentDone: {} });
  });
});
