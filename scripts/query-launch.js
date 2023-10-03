const { SigningArchwayClient } = require("@archwayhq/arch3.js");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { coins } = require("@cosmjs/amino");

require("dotenv").config();

const main = async () => {
  const network = {
    chainId: "constantine-3",
    endpoint: "https://rpc.constantine.archway.tech",
    prefix: "archway",
  };
  // Get wallet and accounts from mnemonic
  const mnemonic = process.env.MNEMONIC;
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: network.prefix,
  });
  const accounts = await wallet.getAccounts();
  const accountAddress = accounts[0].address;
  const client = await SigningArchwayClient.connectWithSigner(
    network.endpoint,
    wallet
  );

  // query
  const nftContractAddress = "archway1zpth3gf320z3m6ylpje57dqpqx2057d0krkcgdgnkfgnwr6gzg2q4am7d5"
  const launchpadContractAddress = "archway1zpth3gf320z3m6ylpje57dqpqx2057d0krkcgdgnkfgnwr6gzg2q4am7d5"

  let msg = {
    get_launch: {
      contract_address: nftContractAddress
    }
  }

  let launch = await client.queryContractSmart(launchpadContractAddress, msg)

  console.log(launch)
};

main();
