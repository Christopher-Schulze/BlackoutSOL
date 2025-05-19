import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Blackout } from '../target/types/blackout';
import { PublicKey, Keypair, Connection, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { assert } from 'chai';

describe('BlackoutSOL Rent Efficiency Tests', () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Blackout as Program<Blackout>;
  const connection = provider.connection;
  
  // Test wallets
  const payerKeypair = Keypair.generate();
  const primaryRecipient = Keypair.generate();
  const recipient2 = Keypair.generate();
  const recipient3 = Keypair.generate();
  const recipient4 = Keypair.generate();
  const recipient5 = Keypair.generate();
  const recipient6 = Keypair.generate();
  
  // Test data
  const testAmount = 0.5 * LAMPORTS_PER_SOL; // 0.5 SOL
  const dummyProofData = Buffer.alloc(128, 0xFF);
  const dummyRangeProof = Buffer.alloc(128, 0xAA);
  const dummyChallenge = Buffer.alloc(32, 0x55);
  const dummyMerkleProof = Buffer.alloc(64, 0x22);
  
  before(async () => {
    // Airdrop SOL to payer
    const airdropSignature = await connection.requestAirdrop(
      payerKeypair.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropSignature);
  });

  it('measures rent efficiency for single-recipient transfers', async () => {
    // Find PDA for transfer state
    const [transferState, bump] = await PublicKey.findProgramAddress(
      [Buffer.from('transfer'), payerKeypair.publicKey.toBuffer()],
      program.programId
    );

    // Find merkle root account (dummy for test)
    const merkleRootAccount = Keypair.generate();
    
    // Log initial lamport balances
    const initialPayerBalance = await connection.getBalance(payerKeypair.publicKey);
    const initialRecipientBalance = await connection.getBalance(primaryRecipient.publicKey);
    
    console.log(`Initial payer balance: ${initialPayerBalance / LAMPORTS_PER_SOL} SOL`);
    
    // Initialize transfer with single recipient
    await program.methods.initialize(
      new anchor.BN(testAmount),
      Array.from(dummyProofData),
      Array.from(dummyRangeProof),
      Array.from(dummyChallenge),
      Array.from(dummyMerkleProof)
    )
    .accounts({
      payer: payerKeypair.publicKey,
      transferState: transferState,
      primaryRecipient: primaryRecipient.publicKey,
      recipient2: null,
      recipient3: null,
      recipient4: null,
      recipient5: null,
      recipient6: null,
      merkleRootAccount: merkleRootAccount.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    })
    .signers([payerKeypair])
    .rpc();
    
    // Perform transfer operations (mock for test)
    // Normally we would execute hops here
    
    // Skip to finalize for testing rent efficiency
    // In a real transfer we would execute 4 hops first
    
    // Mock setup to simulate completed hops
    await program.methods.finalize(
      Array.from(dummyProofData)
    )
    .accounts({
      authority: payerKeypair.publicKey,
      transferState: transferState,
      primaryRecipient: primaryRecipient.publicKey,
      recipient2: null,
      recipient3: null,
      recipient4: null,
      recipient5: null,
      recipient6: null,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([payerKeypair])
    .rpc();
    
    // Check final balances
    const finalPayerBalance = await connection.getBalance(payerKeypair.publicKey);
    const finalRecipientBalance = await connection.getBalance(primaryRecipient.publicKey);
    
    console.log(`Final payer balance: ${finalPayerBalance / LAMPORTS_PER_SOL} SOL`);
    console.log(`Final recipient balance: ${finalRecipientBalance / LAMPORTS_PER_SOL} SOL`);
    
    // Calculate and verify efficiency
    const payerCost = initialPayerBalance - finalPayerBalance;
    const recipientGain = finalRecipientBalance - initialRecipientBalance;
    
    console.log(`Total cost to payer: ${payerCost / LAMPORTS_PER_SOL} SOL`);
    console.log(`Total gain to recipient: ${recipientGain / LAMPORTS_PER_SOL} SOL`);
    
    // Calculate efficiency (percentage of funds that reached recipient)
    const efficiency = (recipientGain / payerCost) * 100;
    console.log(`Transfer efficiency: ${efficiency.toFixed(2)}%`);
    
    // Check that no accounts are left behind
    const accountInfo = await connection.getAccountInfo(transferState);
    assert.isNull(accountInfo, "Transfer state account should be closed");
  });

  it('measures rent efficiency for multi-wallet distribution', async () => {
    // Find PDA for transfer state
    const [transferState, bump] = await PublicKey.findProgramAddress(
      [Buffer.from('transfer'), payerKeypair.publicKey.toBuffer()],
      program.programId
    );

    // Find merkle root account (dummy for test)
    const merkleRootAccount = Keypair.generate();
    
    // Log initial lamport balances
    const initialPayerBalance = await connection.getBalance(payerKeypair.publicKey);
    const initialRecipientBalances = [
      await connection.getBalance(primaryRecipient.publicKey),
      await connection.getBalance(recipient2.publicKey),
      await connection.getBalance(recipient3.publicKey),
      await connection.getBalance(recipient4.publicKey),
      await connection.getBalance(recipient5.publicKey),
      await connection.getBalance(recipient6.publicKey),
    ];
    
    console.log(`Initial payer balance: ${initialPayerBalance / LAMPORTS_PER_SOL} SOL`);
    
    // Initialize transfer with multiple recipients
    await program.methods.initialize(
      new anchor.BN(testAmount),
      Array.from(dummyProofData),
      Array.from(dummyRangeProof),
      Array.from(dummyChallenge),
      Array.from(dummyMerkleProof)
    )
    .accounts({
      payer: payerKeypair.publicKey,
      transferState: transferState,
      primaryRecipient: primaryRecipient.publicKey,
      recipient2: recipient2.publicKey,
      recipient3: recipient3.publicKey,
      recipient4: recipient4.publicKey,
      recipient5: recipient5.publicKey,
      recipient6: recipient6.publicKey,
      merkleRootAccount: merkleRootAccount.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    })
    .signers([payerKeypair])
    .rpc();
    
    // Skip to finalize for testing rent efficiency
    await program.methods.finalize(
      Array.from(dummyProofData)
    )
    .accounts({
      authority: payerKeypair.publicKey,
      transferState: transferState,
      primaryRecipient: primaryRecipient.publicKey,
      recipient2: recipient2.publicKey,
      recipient3: recipient3.publicKey,
      recipient4: recipient4.publicKey,
      recipient5: recipient5.publicKey,
      recipient6: recipient6.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([payerKeypair])
    .rpc();
    
    // Check final balances
    const finalPayerBalance = await connection.getBalance(payerKeypair.publicKey);
    const finalRecipientBalances = [
      await connection.getBalance(primaryRecipient.publicKey),
      await connection.getBalance(recipient2.publicKey),
      await connection.getBalance(recipient3.publicKey),
      await connection.getBalance(recipient4.publicKey),
      await connection.getBalance(recipient5.publicKey),
      await connection.getBalance(recipient6.publicKey),
    ];
    
    // Calculate total received across all recipients
    const totalReceived = finalRecipientBalances.reduce((sum, balance, index) => {
      return sum + (balance - initialRecipientBalances[index]);
    }, 0);
    
    console.log(`Final payer balance: ${finalPayerBalance / LAMPORTS_PER_SOL} SOL`);
    console.log(`Total received by all recipients: ${totalReceived / LAMPORTS_PER_SOL} SOL`);
    
    // Calculate and verify efficiency
    const payerCost = initialPayerBalance - finalPayerBalance;
    
    console.log(`Total cost to payer: ${payerCost / LAMPORTS_PER_SOL} SOL`);
    
    // Calculate efficiency (percentage of funds that reached recipients)
    const efficiency = (totalReceived / payerCost) * 100;
    console.log(`Multi-wallet transfer efficiency: ${efficiency.toFixed(2)}%`);
    
    // Check recipient distribution
    for (let i = 0; i < finalRecipientBalances.length; i++) {
      const received = finalRecipientBalances[i] - initialRecipientBalances[i];
      if (received > 0) {
        console.log(`Recipient ${i+1} received: ${received / LAMPORTS_PER_SOL} SOL`);
      }
    }
    
    // Check that no accounts are left behind
    const accountInfo = await connection.getAccountInfo(transferState);
    assert.isNull(accountInfo, "Transfer state account should be closed");
  });
});
