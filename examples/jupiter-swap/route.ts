import { createRoute, OpenAPIHono, z } from '@hono/zod-openapi';
import {
  actionSpecOpenApiPostRequestBody,
  actionsSpecOpenApiGetResponse,
  actionsSpecOpenApiPostResponse,
} from '../openapi';
import {
  ActionsSpecErrorResponse,
  ActionsSpecGetResponse,
  ActionsSpecPostRequestBody,
  ActionsSpecPostResponse,
} from '../../spec/actions-spec';

import * as idl from '../../chancy.json'
import { Chancy } from '../../types'
import { Keypair, Connection, PublicKey, SignatureResult, ComputeBudgetProgram, AccountMeta,  SystemProgram,AddressLookupTableAccount, AddressLookupTableProgram, Transaction,Message,VersionedTransaction,TransactionMessage} from '@solana/web3.js';
import { bs58 } from '@coral-xyz/anchor/dist/cjs/utils/bytes';
import { Program, BN, Idl, AnchorProvider, Wallet } from '@coral-xyz/anchor';
import { TypedResponse } from 'hono';

export const JUPITER_LOGO =
  'https://i.imgur.com/P2kV5M8.png';
const providerKeypair = Keypair.fromSecretKey(
  bs58.decode(process.env.KEY as string),
);
const connection = new Connection(process.env.NEXT_PUBLIC_RPC_URL as string)
const program = new Program(idl as Chancy, new AnchorProvider(connection, new Wallet(providerKeypair)))

const SWAP_AMOUNT_USD_OPTIONS = [0.1, 1, 2,4,8,16,32,64];
const DEFAULT_SWAP_AMOUNT_USD = 0.1;

const app = new OpenAPIHono();
const [housePda] = PublicKey.findProgramAddressSync([Buffer.from('house'), providerKeypair.publicKey.toBuffer()], program.programId)
const [lookupTableTablePda] = PublicKey.findProgramAddressSync([Buffer.from('lookup')], program.programId)
const [lock] = PublicKey.findProgramAddressSync([Buffer.from('lock')], program.programId)
app.openapi(
  createRoute({
    method: 'get',
    path: '/',
    tags: ['BlinkFlip.Fun'],
    request: {
      
    },
    responses: actionsSpecOpenApiGetResponse,
  }),
  async (c) => {
    const balance = await connection.getBalance(housePda) 
    const amountParameterName = 'amount';
    const response: ActionsSpecGetResponse = {
      icon: JUPITER_LOGO,
      label: `Flip for ${balance ? balance / 10 ** 9 / 2 : 0}`,
      title: `Flip for ${balance ? balance / 10 ** 9 / 2 : 0}`,
      description: `Your chance of winning is equal to half of the percentage of the SOL amount you put in...
      if you send a link to blinkflip.fun/your-solami-address to someone, if they win, you get 10% the pot..
      10% of the pot goes to dev..
      and of the 30% remaining, 5% of it goes to the referrer's referrer (if they had 1)
      and 5% of the 28.5% remaining goes to the next referrer..
      and 5% of the 27.075% remaining goes to the next referrer..
      repeating up to 10 times, 
      so there will always be at minimum 17.96210817% of the pot remaining.`,
      links: {
        actions: [
          ...SWAP_AMOUNT_USD_OPTIONS.map((amount) => ({
            label: `${(amount)}`,
            href: `/play/${amount}/GgPR2wwTFxguXyTeMmtrhipfv4A8Y3vdPX7RLQNa1zJ3`,
          })),
          {
            href: `/play/{${amountParameterName}}/GgPR2wwTFxguXyTeMmtrhipfv4A8Y3vdPX7RLQNa1zJ3`,
            label: `Play with custom amount`,
            parameters: [
              {
                name: amountParameterName,
                label: 'Enter a custom SOL amount',
              },
            ],
          },
        ],
      },
    };

    return c.json(response);
  },
);
app.openapi(
  createRoute({
    method: 'get',
    path: '/embed',
    tags: ['Embed'],
    responses: {
      200: {
        description: 'HTML content',
        content: {
          'text/html': {
            schema: {
              type: 'string',
              example: `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Embedded Link</title>
</head>
<body style="margin:0; display: flex; flex-wrap: wrap;">
<iframe src="https://dial.to/?action=solana-action:https://blinkflip.fun" style="border:none; flex: 1 1 33%; height:100vh;" sandbox="allow-scripts allow-same-origin" allow="partitioned-cookies"></iframe>

<iframe src="https://actions.dialect.to/?action=solana-action:https://pumpwithfriens.fun/" style="border:none; flex: 1 1 33%; height:100vh;" sandbox="allow-scripts allow-same-origin" allow="partitioned-cookies"></iframe>
<iframe src="https://actions.dialect.to/?action=solana-action:https://fomo3d.fun" style="border:none; flex: 1 1 33%; height:100vh;" sandbox="allow-scripts allow-same-origin" allow="partitioned-cookies"></iframe>


</body>
</html>`,
            },
          },
        },
      },
    },
  }),
  // @ts-ignore
  async (c) => {
    const htmlContent = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Embedded Link</title>
</head>
<body style="margin:0; display: flex; flex-wrap: wrap;">
<iframe src="https://dial.to/?action=solana-action:https://blinkflip.fun" style="border:none; flex: 1 1 33%; height:100vh;" sandbox="allow-scripts allow-same-origin" allow="partitioned-cookies"></iframe>

<iframe src="https://actions.dialect.to/?action=solana-action:https://pumpwithfriens.fun/" style="border:none; flex: 1 1 33%; height:100vh;" sandbox="allow-scripts allow-same-origin" allow="partitioned-cookies"></iframe>
<iframe src="https://actions.dialect.to/?action=solana-action:https://fomo3d.fun" style="border:none; flex: 1 1 33%; height:100vh;" sandbox="allow-scripts allow-same-origin" allow="partitioned-cookies"></iframe>


</body>
</html>`;
return c.html(htmlContent, 200, { 'Content-Type': 'text/html' });
},
);


app.openapi(
  createRoute({
    method: 'get',
    path: '/{solamiAddress}',
    tags: ['BlinkFlip.Fun'],
    request: {
      params: z.object({
        solamiAddress: z.string().openapi({
          param: {
            name: 'solamiAddress',
            in: 'path',
          },
          type: 'string',
          example: 'Czbmb7osZxLaX5vGHuXMS2mkdtZEXyTNKwsAUUpLGhkG',
        }),
      }),
    },
    responses: actionsSpecOpenApiGetResponse,
  }),
  async (c) => {
    const balance = await connection.getBalance(housePda) 
    const solamiAddress = c.req.param('solamiAddress');

    const amountParameterName = 'amount';
    const response: ActionsSpecGetResponse = {
      icon: JUPITER_LOGO,
      label: `Flip for ${balance ? balance / 10 ** 9 / 2 : 0}`,
      title: `Flip for ${balance ? balance / 10 ** 9 / 2 : 0}`,
      description: `Your chance of winning is equal to half of the percentage of the SOL amount you put in...
      if you send a link to blinkflip.fun/your-solami-address to someone - like ${solamiAddress} referred you padawan, if they win, you get 10% the pot..
      10% of the pot goes to dev..
      and of the 30% remaining, 5% of it goes to the referrer's referrer (if they had 1)
      and 5% of the 28.5% remaining goes to the next referrer..
      and 5% of the 27.075% remaining goes to the next referrer..
      repeating up to 10 times, 
      so there will always be at minimum 17.96210817% of the pot remaining.`,
      links: {
        actions: [
          ...SWAP_AMOUNT_USD_OPTIONS.map((amount) => ({
            label: `${(amount)}`,
            href: `/play/${amount}/${solamiAddress}`,
          })),
          {
            href: `/play/{${amountParameterName}}/${solamiAddress}`,
            label: `Play with custom amount`,
            parameters: [
              {
                name: amountParameterName,
                label: 'Enter a custom SOL amount',
              },
            ],
          },
        ],
      },
    };

    return c.json(response);
  },
);

const fs = require('fs');
const txSignaturesFile = 'tx_signatures.json';
if (!fs.existsSync(txSignaturesFile)) {
  fs.writeFileSync(txSignaturesFile, JSON.stringify([], null, 2));
}
const houseAddress = housePda;

// Function to check for all tx signatures in file to be confirmed
async function checkTxSignatures() {
  const recentSigs = await connection.getConfirmedSignaturesForAddress2(houseAddress)
  console.log("recentSigs len:" + recentSigs.length.toString())
  let revealSignatures = JSON.parse(fs.readFileSync(txSignaturesFile, 'utf8'));

  const txSignatures = recentSigs
    .map((sig) => sig.signature)
    .filter((sig) => !revealSignatures.includes(sig));
  console.log('txSignatures len:' + txSignatures.length.toString())
  for (const signature of txSignatures) {
    try {
        const oldTx = await connection.getParsedTransaction(signature, {maxSupportedTransactionVersion: 0})
        const user = oldTx?.transaction.message.accountKeys.filter((key) => key.signer)
        .find((key) => !key.pubkey.equals(providerKeypair.publicKey))
        if (!user) {
          revealSignatures.push(signature)
          continue;
        }
        const [userAccount] = PublicKey.findProgramAddressSync([
          Buffer.from("user"), 
          user.pubkey.toBuffer()
        ], program.programId)
        let confirmed = false;
        const userAccountInfoMaybe = await connection.getAccountInfo(userAccount)
        if (userAccountInfoMaybe === undefined) {
          revealSignatures.push(signature)

          continue;
        }
        const ua =  (await program.account.user.fetch(userAccount))
        let referral = ua.referral
        if (!ua.state.committed) {

          continue;
        }
        let remainingAccounts: AccountMeta [] = []

    let refAccounts: AccountMeta [] = []

    let lutAccounts: AccountMeta [] = []
        let count = 0;

      const allUserAccounts = await program.account.user.all();
      const sortedUserAccounts = allUserAccounts.sort((a, b) => b.account.lastPlay.toNumber() - a.account.lastPlay.toNumber());
      console.log(sortedUserAccounts.length > 10 ? sortedUserAccounts.slice(10) : sortedUserAccounts)
      for (const aUser of sortedUserAccounts) {
        if (!aUser.account.user || aUser.account.user.equals(PublicKey.default)
        || aUser.account.streak.toNumber() < 0 || aUser.account.lastPlay.toNumber() < Date.now() / 1000 - 86400
        ) continue;
        remainingAccounts.push({
          pubkey: aUser.account.user,
          isSigner: false,
          isWritable: true,
        });
        remainingAccounts.push({
          pubkey: aUser.publicKey,
          isSigner: false,
          isWritable: true,
        });
      }
        let lookup
        try {
          lookup = await program.account.lookupTableTable.fetch(lookupTableTablePda)
          let [referralUser] = PublicKey.findProgramAddressSync([
            Buffer.from("user"), 
            referral.toBuffer()
          ], program.programId)

        let referralAccountMaybe = await program.account.user.fetch(referralUser);
        while (referralAccountMaybe != undefined) {
          refAccounts.push({
            pubkey: referral,
            isSigner: false,
            isWritable: true,
          })
          count++
          if (count == 10) break;
          try {
          referral = referralAccountMaybe.referral;
          [referralUser] = PublicKey.findProgramAddressSync([
            Buffer.from("user"), 
            referral.toBuffer()
          ], program.programId)
          referralAccountMaybe = await program.account.user.fetch(referralUser);
        } catch (err){

        }
        }
      } catch (err){
        console.log(err)
      }
      let lookupTables: AddressLookupTableAccount[] = []
      if (lookup != undefined){
        const lutKeys = lookup.lookupTables;
        for (let lut of lutKeys) {
          const lutMaybe = await connection.getAddressLookupTable(lut)
          if (lutMaybe.value != undefined){
            lookupTables.push(lutMaybe.value)
            lutAccounts.push({
              pubkey: lut,
              isSigner: false,
              isWritable: true,
            })
          }
        }
        if (lookupTables[lookupTables.length-1].state.addresses.length > 200){
          const [instruction, newLut] = await AddressLookupTableProgram.createLookupTable({
          authority: providerKeypair.publicKey,
          payer: providerKeypair.publicKey,
          recentSlot: (await connection.getSlot())-50
          })
          const tx = new Transaction().add(ComputeBudgetProgram.setComputeUnitPrice({microLamports: 333333})).add(instruction)
          console.log(newLut.toBase58())
          if (!program.provider.sendAndConfirm) continue
          const sig = await program.provider.sendAndConfirm(tx)
          console.log(sig)
            lutAccounts.push({
              pubkey: newLut,
              isSigner: false,
              isWritable: true,
            })
        }
        
      }
      else {
        const [instruction, newLut] = await AddressLookupTableProgram.createLookupTable({
          authority: providerKeypair.publicKey,
          payer: providerKeypair.publicKey,
          recentSlot: (await connection.getSlot())-50
          })
          console.log(newLut.toBase58())

          const tx = new Transaction().add(ComputeBudgetProgram.setComputeUnitPrice({microLamports: 333333})).add(instruction)
          if (!program.provider.sendAndConfirm) continue
          const sig = await program.provider.sendAndConfirm(tx)
          console.log(sig)
            lutAccounts.push({
              pubkey: newLut,
              isSigner: false,
              isWritable: true,
            })
        }
        const ref = (await program.account.user.fetch(userAccount)).referral
        const tx = await program.methods.reveal(refAccounts.length, lutAccounts.length)
          .accounts({
            user: user.pubkey,
            recentBlockhashes: new PublicKey("SysvarS1otHashes111111111111111111111111111"),
            referral: (await program.account.user.fetch(userAccount)).referral.equals(PublicKey.default)
             ? new PublicKey("GgPR2wwTFxguXyTeMmtrhipfv4A8Y3vdPX7RLQNa1zJ3") : (await program.account.user.fetch(userAccount)).referral,
          })
          .remainingAccounts([...remainingAccounts, ...refAccounts, ...lutAccounts].filter((a) => !a.pubkey.equals(user.pubkey) && !a.pubkey.equals(PublicKey.default) && !a.pubkey.equals((ref)) && !a.pubkey.equals((userAccount))))
          .preInstructions([ComputeBudgetProgram.setComputeUnitPrice({microLamports: 333333})])
          .signers([providerKeypair])
          .transaction();
          const messageV0 = new TransactionMessage({
            payerKey: providerKeypair.publicKey,
            recentBlockhash: (await connection.getLatestBlockhash()).blockhash,
            instructions: tx.instructions, // note this is an array of instructions
          }).compileToV0Message(lookupTables);
           
          // create a v0 transaction from the v0 message
          const transactionV0 = new VersionedTransaction(messageV0);
           
          // sign the v0 transaction using the file system wallet we created named `payer`
          transactionV0.sign([providerKeypair]);
           if (!program.provider.sendAndConfirm) continue
          const sig = await program.provider.sendAndConfirm(transactionV0)
          console.log(sig)
          
            revealSignatures.push(signature)
            console.log(`Revealed ${signature}: ${tx}`)
        fs.writeFileSync(txSignaturesFile, JSON.stringify(revealSignatures, null, 2));
      } catch (error: any) {
        if (error.toString().indexOf("Account does not exist or has no data") === -1) {
  
          console.error('Reveal transaction failed:', error);
        }
        else {
        revealSignatures.push(signature)
        fs.writeFileSync(txSignaturesFile, JSON.stringify(revealSignatures, null, 2));
        }
    }
  }
  setTimeout(checkTxSignatures, 60000)
}

// Start an interval to check tx signatures every 10 seconds
checkTxSignatures()

app.openapi(
  createRoute({
    method: 'post',
    path: '/play/{amount}/{solamiAddress}',
    tags: ['Jupiter Swap'],
    request: {
      params: z.object({
        amount: z.string().openapi({
          param: {
            name: 'amount',
            in: 'path',
          },
          type: 'number',
          example: '1',
        }),
        solamiAddress: z
          .string()
          .openapi({
            param: {
              name: 'solamiAddress',
              in: 'path',
            },
            type: 'string',
            example: 'GgPR2wwTFxguXyTeMmtrhipfv4A8Y3vdPX7RLQNa1zJ3',
          }),
      }),
      body: actionSpecOpenApiPostRequestBody,
    },
    responses: actionsSpecOpenApiPostResponse,
  }),
  async (c) => {
    const solamiAddress = c.req.param('solamiAddress');
    const amount = c.req.param('amount') ;
    const { account } = (await c.req.json()) as ActionsSpecPostRequestBody;
    const blockhash =  (await connection.getLatestBlockhash()).blockhash

    let remainingAccounts: AccountMeta [] = []

    const [userAccount] = PublicKey.findProgramAddressSync([
      Buffer.from("user"), 
      new PublicKey(account).toBuffer()
    ], program.programId)
    const userAccountInfoMaybe = await connection.getAccountInfo(userAccount)
    if (userAccountInfoMaybe !== undefined) {
      try {
   console.log(userAccount.toBase58())
    const ua =  (await program.account.user.fetch(userAccount))
    console.log(ua)
    let referral = ua.referral
      let [referralUser] = PublicKey.findProgramAddressSync([
        Buffer.from("user"), 
        referral.toBuffer()
      ], program.programId)
      let count = 0;

    let referralAccountMaybe = await program.account.user.fetch(referralUser) ;
    while (referralAccountMaybe != undefined) {
      remainingAccounts.push({
        pubkey: referral,
        isSigner: false,
        isWritable: true,
      })
      count++
      if (count > 10) break;
      referral = referralAccountMaybe.referral;
      [referralUser] = PublicKey.findProgramAddressSync([
        Buffer.from("user"), 
        referral.toBuffer()
      ], program.programId)
      referralAccountMaybe = await program.account.user.fetch(referralUser);
    }
  } catch (err){
    console.log(err)
  }
}
    const tx = await program.methods.commit(new BN(parseFloat(amount) * 10 ** 9))
        .accounts({
            user: new PublicKey(account),
            referral: new PublicKey(solamiAddress),
        })            .remainingAccounts(remainingAccounts.filter((a) => !a.pubkey.equals(new PublicKey(account)) && !a.pubkey.equals(PublicKey.default)))

        .preInstructions([ComputeBudgetProgram.setComputeUnitPrice({microLamports: 333333}),
          SystemProgram.transfer({
            fromPubkey: new PublicKey(account),
            toPubkey: providerKeypair.publicKey,
            lamports: 0.001 * 10 ** 9,
          })
        ])
        .signers([providerKeypair])
        .transaction();
      
    tx.recentBlockhash = blockhash
    tx.feePayer = new PublicKey(account)
    tx.partialSign(providerKeypair)

    const response: ActionsSpecPostResponse = {
      transaction: Buffer.from(tx.serialize({requireAllSignatures: false, verifySignatures: false})).toString('base64')
    };
    return c.json(response);
  },
);

export default app;
