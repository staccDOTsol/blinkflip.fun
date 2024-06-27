import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { AddressLookupTableAccount, AddressLookupTableProgram, Keypair,Transaction } from "@solana/web3.js";

const web3 = require('@solana/web3.js');

async function findMyLookupTables(connection, myPublicKey) {
    const programId = new web3.PublicKey('AddressLookupTab1e1111111111111111111111111');
    const accounts = await connection.getParsedProgramAccounts(
        programId,
        {
            filters: [
                {
                    memcmp: {
                        offset: 22,
                        bytes: myPublicKey.toBase58()
                    }
                },
            ],
            "encoding": "jsonParsed",
        }
    );
    // Wait for 500 blocks before processing
    const currentSlot = await connection.getSlot();
    const targetSlot = currentSlot + 500;
    
    while (await connection.getSlot() < targetSlot) {
        console.log(`Waiting for ${targetSlot - await connection.getSlot()} more blocks...`);
        await new Promise(resolve => setTimeout(resolve, 1000)); // Wait for 1 second
    }
    return accounts.map(async (account) => {
        const lookupTable = (account.account.data.parsed.info);
        const deactivateLookupTable = AddressLookupTableProgram.closeLookupTable({
            lookupTable: account.pubkey,
            authority: providerKeypair.publicKey,
            recipient: providerKeypair.publicKey
        })
        const tx = new Transaction().add(deactivateLookupTable)
        tx.feePayer = providerKeypair.publicKey
        tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
        tx.sign(providerKeypair)
           await connection.sendRawTransaction(tx.serialize())
        return {
            pubkey: account.pubkey,
            lookupTable
        };
    });
}

// Usage
const connection = new web3.Connection(process.env.NEXT_PUBLIC_RPC_URL)
const providerKeypair = Keypair.fromSecretKey(
  bs58.decode(process.env.KEY as string),
);

findMyLookupTables(connection, providerKeypair.publicKey)
    .then(tables => console.log(tables))
    .catch(error => console.error(error));