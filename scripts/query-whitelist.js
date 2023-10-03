const { SigningArchwayClient } = require("@archwayhq/arch3.js");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { coins } = require("@cosmjs/amino");
const { MerkleTree } = require("merkletreejs");
const fs = require("fs");
const sha256 = require("crypto-js/sha256");


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

  // whitelist merkle generation

  const wlAccounts = JSON.parse(fs.readFileSync("./testdata/wl_list.json").toString());

  const leaves = wlAccounts.map((a) => sha256(a.address));
  const tree = new MerkleTree(leaves, sha256, { sort: true })

  const whitelist_merkle_root = tree.getHexRoot().replace('0x', '')

  if (whitelist_merkle_root != launch.whitelist_merkle_root) {
    throw new Error("Whitelist merkle root is not the same")
  }

  function getMerkleProof({
    address,
  }) {
    return tree.getHexProof(sha256(address).toString())
      .map((v) => v.replace('0x', ''));
  }

  const accountTest = "archway1eawplys3a5vxswxyddm420vd5su4jzdae9eyy3"

  msg = {
    get_whitelist_status: {
      contract_address: nftContractAddress,
      account_address: accountTest,
      proof: getMerkleProof({ address: accountTest })
    }
  }

  let is_whitelist = await client.queryContractSmart(launchpadContractAddress, msg)

  console.log(launch)
  console.log("Is whitelist?", is_whitelist)
};

main();
