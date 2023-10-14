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

  // execute
  const launchpadContractAddress = "archway1zpth3gf320z3m6ylpje57dqpqx2057d0krkcgdgnkfgnwr6gzg2q4am7d5"
  const nftContractAddress = "archway1zpth3gf320z3m6ylpje57dqpqx2057d0krkcgdgnkfgnwr6gzg2q4am7d5"
  const max_supply = 1000
  const base_uri = "https://ipfs.io/ipfs/QmejYa4kkcnCjDiZwy2YnNCY2CBBYnnxDV3V2F1Eh77iya"
  const is_base_uri_static = false
  const media_extension = "png"
  const whitelist_price = coin(0, "aconst")
  const whitelist_max_buy = 0
  const whitelist_started_at = "0"
  const whitelist_ended_at = "100000000000"
  const public_price = coin(0, "aconst")
  const public_max_buy = 1
  const public_started_at = "0"
  const public_ended_at = "0"
  const royalty_percentage = null
  const royalty_payment_address = null

  // whitelist merkle generation

  const wlAccounts = JSON.parse(fs.readFileSync("./testdata/wl_list.json").toString());

  const leaves = wlAccounts.map((a) => sha256(a.address));
  const tree = new MerkleTree(leaves, sha256, { sort: true })

  const whitelist_merkle_root = tree.getHexRoot().replace('0x', '')

  const msg = {
    add_launch: {
      owner_address: accountAddress,
      contract_address: nftContractAddress,
      max_supply,
      base_uri,
      is_base_uri_static,
      media_extension,
      whitelist_price,
      whitelist_max_buy,
      whitelist_started_at,
      whitelist_ended_at,
      public_price,
      public_max_buy,
      public_started_at,
      public_ended_at,
      royalty_percentage,
      royalty_payment_address,
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