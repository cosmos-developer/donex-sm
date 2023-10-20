import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { DonexClient } from "./Donex.client";
import { Decimal } from "@cosmjs/math";
import { Addr } from "./Donex.types";
import { coins } from "@cosmjs/amino";
const rpcEndpoint =
  "http://ec2-3-0-52-194.ap-southeast-1.compute.amazonaws.com:26657";
const contract_addr =
  "comdex1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqdfklyz";

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
    rpcEndpoint,
    signer,
    {
      gasPrice: { amount: Decimal.fromUserInput("1000", 0), denom: "ucmdx" },
    }
  );

  let donex = new DonexClient(client, accounts[0].address, contract_addr);
  return donex;
}
async function submitSocial(client: DonexClient) {
  let result = await client.submitSocial({
    address: "comdex1elk425naxzh895xaedl4q95zylag0d7j08yhd2",
    socialInfo: ["facebook", "123"],
  });

  console.log(result);
}
async function getSocialsByAddress(client: DonexClient) {
  let result = await client.getSocialsByAddress({
    address: "comdex1elk425naxzh895xaedl4q95zylag0d7j08yhd2",
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
  //   // Submit transaction
  //   (async () => {
  //     let client = await createClient();
  //     await submitSocial(client);
  //     // console.log(result)
  //   })();
  //   // Query information from contract
  //   (async () => {
  //     let client = await createClient();
  //     let result = await getSocialsByAddress(client);
  //     console.log(result);
  //   })();

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
