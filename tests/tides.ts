import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Tides } from "../target/types/tides";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("tides", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Tides as Program<Tides>;
  const gameStateKeypair = Keypair.generate();
  const playerKeypair = Keypair.generate();

  it("Initializes the game", async () => {
    const currencyMint = Keypair.generate().publicKey;
    const serverSigner = Keypair.generate().publicKey;
    const maxPlayersPerShard = new anchor.BN(1000);

    const tx = await program.methods
      .initialize(currencyMint, serverSigner, maxPlayersPerShard)
      .accounts({
        gameState: gameStateKeypair.publicKey,
        admin: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([gameStateKeypair])
      .rpc();

    const gameState = await program.account.gameState.fetch(
      gameStateKeypair.publicKey
    );

    expect(gameState.currencyMint.toString()).to.equal(currencyMint.toString());
    expect(gameState.admin.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(gameState.paused).to.be.false;
    expect(gameState.maxPlayersPerShard.toNumber()).to.equal(1000);
  });

  it("Registers a player", async () => {
    const shard = 0;
    const mapId = new anchor.BN(1);

    const [shardData] = PublicKey.findProgramAddressSync(
      [Buffer.from("shard"), Buffer.from([shard])],
      program.programId
    );

    const tx = await program.methods
      .registerPlayer(shard, mapId)
      .accounts({
        gameState: gameStateKeypair.publicKey,
        playerState: Keypair.generate().publicKey,
        shardData: shardData,
        player: playerKeypair.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([playerKeypair])
      .rpc();

    // Add assertions here
  });
});

