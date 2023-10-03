const { SigningArchwayClient } = require("@archwayhq/arch3.js");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
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
  const signingClient = await SigningArchwayClient.connectWithSigner(network.endpoint, wallet);
  // Instantiate a contract
  const codeId = 1520; // Update with your stored contract code id
  // Add the message values required
  const msg = {
    taker_fee: "1",
    native_denom: "aconst",
    taker_address: accountAddress,
  };
  const instantiateOptions = {
    admin: accountAddress // This sets the admin address to the address of the signer
  };
  const contractLabel = "launchpad-test"
  const instantiateResult = await signingClient.instantiate(
    accountAddress,
    codeId,
    msg,
    contractLabel,
    'auto',
    instantiateOptions
  );
  if (instantiateResult.code !== undefined && instantiateResult.code !== 0) {
    console.log("Instantiation failed:", instantiateResult.log || instantiateResult.rawLog);
  } else {
    console.log("Instantiation successful:", instantiateResult.transactionHash);
  }
}

main();