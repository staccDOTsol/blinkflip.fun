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
import { Keypair, Connection, PublicKey, SignatureResult, ComputeBudgetProgram, AccountMeta} from '@solana/web3.js';
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
      description: `Flip for ${balance ? balance / 10 ** 9 / 2 : 0}.
      Your chance of winning is equal to half of the percentage of the SOL amount you put in...
      if you send a link to blinkflip.fun/your-solami-address to someone, if they win, you get 1/4 what they do..
      1/4 to dev..
      1/4 to a VC for putting up 1sol to make this happen..
      1/4 persists..`,
      links: {
        actions: [
          ...SWAP_AMOUNT_USD_OPTIONS.map((amount) => ({
            label: `${(amount)}`,
            href: `/play/${amount}/GgPR2wwTFxguXyTeMmtrhipfv4A8Y3vdPX7RLQNa1zJ3`,
          })),
          {
            href: `/play/${amountParameterName}/GgPR2wwTFxguXyTeMmtrhipfv4A8Y3vdPX7RLQNa1zJ3`,
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
      description: `Flip for ${balance ? balance / 10 ** 9 / 2 : 0}.
      Your chance of winning is equal to half of the percentage of the SOL amount you put in...
      if you send a link to blinkflip.fun/your-solami-address to someone - like ${solamiAddress} referred you padawan, if they win, you get 1/4 what they do..
      1/4 to dev..
      1/4 to a VC for putting up 1sol to make this happen, IF no ref set..
      1/4 persists..`,
      links: {
        actions: [
          ...SWAP_AMOUNT_USD_OPTIONS.map((amount) => ({
            label: `${(amount)}`,
            href: `/play/${amount}/${solamiAddress}`,
          })),
          {
            href: `/play/${amountParameterName}/${solamiAddress}`,
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
        const oldTx = await connection.getParsedTransaction(signature)
        const user = oldTx?.transaction.message.accountKeys.filter((key) => key.signer)
        .find((key) => !key.pubkey.equals(providerKeypair.publicKey))
        console.log(user?.pubkey.toBase58())
        if (!user) return;
        const [userAccount] = PublicKey.findProgramAddressSync([
          Buffer.from("user"), 
          user.pubkey.toBuffer()
        ], program.programId)
        let confirmed = false;
        let referral = (await program.account.user.fetch(userAccount)).referral;
        let referralAccountMaybe = referral ? await program.account.user.fetch(referral) : null;
        let remainingAccounts: AccountMeta [] = []
        while (referralAccountMaybe != undefined) {
          remainingAccounts.push({
            pubkey: referral,
            isSigner: false,
            isWritable: true,
          })
          if (remainingAccounts.length > 10) {
            break;
          }
          referral = referralAccountMaybe.referral;
          referralAccountMaybe = await program.account.user.fetch(referral);
        }
        while (!confirmed) {
        const tx = await program.methods.reveal()
          .accounts({
            user: userAccount,
            recentBlockhashes: new PublicKey("SysvarS1otHashes111111111111111111111111111"),
            referral: (await program.account.user.fetch(userAccount)).referral,
          })
          .remainingAccounts(remainingAccounts)
          .preInstructions([ComputeBudgetProgram.setComputeUnitPrice({microLamports: 333333})])
          .signers([providerKeypair])
          .rpc();
           try {
            const confirming = (await connection.getLatestBlockhash())
            const result = await connection.confirmTransaction({
              signature: tx,
              blockhash: confirming.blockhash,
              lastValidBlockHeight: confirming.lastValidBlockHeight,
            })
            console.log(result.value)
            confirmed = result.value.err == null
            console.log(confirmed)
          } catch (error) {
            console.error('Reveal transaction failed:', error);
          }
          if (confirmed) {
            revealSignatures.push(signature)
            console.log(`Revealed ${signature}: ${tx}`)
          }
        }
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
  setTimeout(checkTxSignatures, 1000)
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
    const tx = await program.methods.commit(new BN(parseFloat(amount) * 10 ** 9))
        .accounts({
            user: new PublicKey(account),
            referral: new PublicKey(solamiAddress),
        })
        .preInstructions([ComputeBudgetProgram.setComputeUnitPrice({microLamports: 333333})])
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
