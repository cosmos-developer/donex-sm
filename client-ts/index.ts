import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { DonexClient } from "./Donex.client";
import { Decimal } from "@cosmjs/math";
import { Addr } from "./Donex.types";
import { coins } from "@cosmjs/amino";
const RPC_ENDPOINT =
  "http://ec2-3-0-52-194.ap-southeast-1.compute.amazonaws.com:26657";
const CONTRACT_ADDRESS =
  "comdex17p9rzwnnfxcjp32un9ug7yhhzgtkhvl9jfksztgw5uh69wac2pgs4jg6dx";

async function createClient() {
  // replace with keplr signer here
  const signer = await DirectSecp256k1HdWallet.fromMnemonic(
    "hover oyster chief slice eye police wrestle price syrup present drastic bone rally other subway away august renew drop parrot situate nation scatter venue",
    {
      prefix: "comdex",
    }
  );
  let accounts = await signer.getAccounts();
  // end replace with keplr signer

  let client = await SigningCosmWasmClient.connectWithSigner(
    RPC_ENDPOINT,
    signer,
    {
      gasPrice: { amount: Decimal.fromUserInput("1000", 0), denom: "ucmdx" },
    }
  );

  let donex = new DonexClient(client, accounts[0].address, CONTRACT_ADDRESS);
  return donex;
}

// Only callable by contract owner
/**
 *
 * @param client : DonexClient instance
 * @param address : user address
 * @param socialInfo : user social infos(in form of [platform, profile_id])
 */
async function submitSocial(
  client: DonexClient,
  address: Addr,
  socialInfo: string[]
) {
  let result = await client.submitSocial({
    address: address,
    socialInfo: socialInfo,
  });

  console.log(result);
}

async function getSocialsByAddress(client: DonexClient, address: Addr) {
  let result = await client.getSocialsByAddress({
    address: address,
  });

  return result.social_infos;
}

async function getAddressesBySocial(
  client: DonexClient,
  platform: string,
  profileId: string
) {
  let result = await client.getAddressesBySocial({ platform, profileId });
  return result.address;
}

async function sendDonate(
  client: DonexClient,
  recipient: Addr,
  amount: number,
  denom: string
) {
  let result = await client.donate(
    { recipient },
    "auto",
    "",
    coins(amount, denom)
  );
  return result;
}
try {
  // Submit transaction
  // (async () => {
  //   let client = await createClient();
  //   await submitSocial(
  //     client,
  //     "comdex1elk425naxzh895xaedl4q95zylag0d7j08yhd2",
  //     ["google", "456"]
  //   );
  //   // console.log(result)
  // })();
  // // Query information from contract
  // (async () => {
  //   let client = await createClient();
  //   let result = await getSocialsByAddress(
  //     client,
  //     "comdex1elk425naxzh895xaedl4q95zylag0d7j08yhd2"
  //   );
  //   console.log(result);
  // })();

  // Query address, and then send donate
  (async function () {
    let client = await createClient();
    let result = await getAddressesBySocial(client, "facebook", "123");
    console.log(result);
    let result2 = await sendDonate(client, result[0], 1000000, "ucmst");
    console.log(result2);
  })();
} catch (e) {
  console.log(e);
}
