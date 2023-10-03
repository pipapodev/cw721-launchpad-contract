const { SigningArchwayClient } = require("@archwayhq/arch3.js");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { MerkleTree } = require("merkletreejs");
const fs = require("fs");
const sha256 = require("crypto-js/sha256");
const { coin } = require('@cosmjs/amino');

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

  const launchpadContractAddress = "archway1zpth3gf320z3m6ylpje57dqpqx2057d0krkcgdgnkfgnwr6gzg2q4am7d5"
  const nftContractAddress = "archway1zpth3gf320z3m6ylpje57dqpqx2057d0krkcgdgnkfgnwr6gzg2q4am7d5"
  // whitelist merkle generation
  const wlAccounts = JSON.parse(fs.readFileSync("./testdata/wl_list.json").toString());

  const leaves = wlAccounts.map((a) => sha256(a.address));
  const tree = new MerkleTree(leaves, sha256, { sort: true })

  const whitelist_merkle_root = tree.getHexRoot().replace('0x', '')

  const msg = {
    modify_launch: {
      contract_address: nftContractAddress,
      whitelist_merkle_root
    }
  };

  console.log(msg);

  const { transactionHash } = await client.execute(
    accountAddress,
    launchpadContractAddress,
    msg,
    "auto"
  );

  console.log("Transaction Hash: ", transactionHash);

}

main();