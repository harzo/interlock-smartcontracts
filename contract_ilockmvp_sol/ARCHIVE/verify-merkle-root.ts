import { program } from 'commander'
import fs from 'fs'
import { BigNumber, utils } from 'ethers'

program
  .version('0.0.0')
  .requiredOption(
    '-i, --input <path>',
    'input JSON file location containing the merkle proofs for each account and the merkle root'
  )

const json = JSON.parse(fs.readFileSync(process.argv[3], { encoding: 'utf8' }))

const combinedHash = (first: Buffer, second: Buffer): Buffer => {
  if (!first) {
    return second
  }
  if (!second) {
    return first
  }

  return Buffer.from(
    utils.solidityKeccak256(['bytes32', 'bytes32'], [first, second].sort(Buffer.compare)).slice(2),
    'hex'
  )
}

const toNode = (index: number | BigNumber, account: string, share: BigNumber, owes: BigNumber, pool: BigNumber): Buffer => {
  const pairHex = utils.solidityKeccak256(['uint256', 'address', 'uint256', 'uint256', 'uint256'], [index, account, share, owes, pool])
  return Buffer.from(pairHex.slice(2), 'hex')
}

const verifyProof = (
  index: number | BigNumber,
  account: string,
  share: BigNumber,
  owes: BigNumber,
  pool: BigNumber,
  proof: Buffer[],
  root: Buffer
): boolean => {
  let pair = toNode(index, account, share, owes, pool)
  for (const item of proof) {
    pair = combinedHash(pair, item)
  }

  return pair.equals(root)
}

const getNextLayer = (elements: Buffer[]): Buffer[] => {
  return elements.reduce<Buffer[]>((layer, el, idx, arr) => {
    if (idx % 2 === 0) {
      // Hash the current element with its pair element
      layer.push(combinedHash(el, arr[idx + 1]))
    }

    return layer
  }, [])
}

const getRoot = (balances: { account: string; share: BigNumber; owes: BigNumber; pool: BigNumber; index: number }[]): Buffer => {
  let nodes = balances
    .map(({ account, share, owes, pool, index }) => toNode(index, account, share, owes, pool))
    // sort by lexicographical order
    .sort(Buffer.compare)

  // deduplicate any eleents
  nodes = nodes.filter((el, idx) => {
    return idx === 0 || !nodes[idx - 1].equals(el)
  })

  const layers = []
  layers.push(nodes)

  // Get next layer until we reach the root
  while (layers[layers.length - 1].length > 1) {
    layers.push(getNextLayer(layers[layers.length - 1]))
  }

  return layers[layers.length - 1][0]
}

if (typeof json !== 'object') throw new Error('Invalid JSON')

const merkleRootHex = json.merkleRoot
const merkleRoot = Buffer.from(merkleRootHex.slice(2), 'hex')

let balances: { index: number; account: string; share: BigNumber; owes: BigNumber; pool: BigNumber }[] = []
let valid = true

Object.keys(json.claims).forEach((address) => {
  const claim = json.claims[address]
  const proof = claim.proof.map((p: string) => Buffer.from(p.slice(2), 'hex'))
  balances.push({ index: claim.index, account: address, share: BigNumber.from(claim.share), owes: BigNumber.from(claim.owes), pool: BigNumber.from(claim.pool)})
  if (verifyProof(claim.index, address, claim.share, claim.owes, claim.pool, proof, merkleRoot)) {
    console.log('Verified proof for', claim.index, address)
  } else {
    console.log('Verification for', address, 'failed')
    valid = false
  }
})

if (!valid) {
  console.error('Failed validation for 1 or more proofs')
  process.exit(1)
}
console.log('Done!')

// Root
const root = getRoot(balances).toString('hex')
console.log('Reconstructed merkle root', root)
console.log('Root matches the one read from the JSON?', root === merkleRootHex.slice(2))
