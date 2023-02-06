import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SafeExchange } from "../target/types/safe_exchange";

describe("safe_exchange", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SafeExchange as Program<SafeExchange>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
