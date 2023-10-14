const { SigningArchwayClient } = require("@archwayhq/arch3.js");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { coins } = require('@cosmjs/amino');

require("dotenv").config();

const main = async () => {
  const network = {
    chainId: 'constantine-3',
    endpoint: 'https://rpc.constantine.archway.tech',
    prefix: 'archway',
  };
  // Get wallet and accounts from mnemonic
  const mnemonic = process.env.MNEMONIC;
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, { prefix: network.prefix });
  const accounts = await wallet.getAccounts();
  const accountAddress = accounts[0].address;
  const client = await SigningArchwayClient.connectWithSigner(network.endpoint, wallet);


  // execute
  const launchpadContractAddress = process.env.LAUNCHPAD_CONTRACT_ADDRESS
  const nftContractAddress = "archway14c2hxvljtwpl39ev0cwtm77jg2judeddwxjhstrj5avceplwxqrsmqfxhu"
  const public_price = "0"
  console.log(accountAddress)

  const msg = {
    mint: {
      contract_address: nftContractAddress,
      receiver_address: null
    }
  };

  const { transactionHash } = await client.execute(
    accountAddress,
    launchpadContractAddress,
    msg,
    "auto",
    undefined,
    coins(2, "aconst")
  );

  console.log("Transaction Hash: ", transactionHash);

}

main();