import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { DonexClient } from "./Donex.client";
import { Decimal } from "@cosmjs/math";

const rpcEndpoint = "http://ec2-3-0-52-194.ap-southeast-1.compute.amazonaws.com:26657"
const contract_addr = "comdex14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9spunaxy"

async function createClient() {
    // replace with keplr signer here
    const signer = await DirectSecp256k1HdWallet.fromMnemonic(
        "hover oyster chief slice eye police wrestle price syrup present drastic bone rally other subway away august renew drop parrot situate nation scatter venue",
        {
            prefix: "comdex"
        }
    )
    let accounts = await signer.getAccounts()
    // end replace with keplr signer

    let client = await SigningCosmWasmClient.connectWithSigner(
        rpcEndpoint, 
        signer,
        {
            gasPrice: {amount: Decimal.fromUserInput("1000", 0), denom: "ucmdx"}
        }
    )

    let donex = new DonexClient(client, accounts[0].address, contract_addr)
9
    let result = await donex.submitSocial({
        address: "comdex1elk425naxzh895xaedl4q95zylag0d7j08yhd2",
        socialInfo: ["facebook", "123"]
    })

    console.log(result)
}

try {
    createClient()
} catch {

}