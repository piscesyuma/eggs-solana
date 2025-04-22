import bs58 from 'bs58';
import { Keypair } from '@solana/web3.js';

describe("keygen", () => {
  it("should generate a new keypair", () => {
    // Your private key array from id.json
    const privateKeyArray: number[] = [157,240,92,76,195,141,93,96,254,82,125,255,121,252,119,222,204,237,129,1,29,220,187,136,192,180,151,220,183,128,142,249,106,229,182,154,1,30,129,103,127,143,233,25,142,116,81,183,178,109,187,36,90,196,130,120,226,191,179,212,93,75,47,135];

    // Convert the array to a Uint8Array
    const privateKeyUint8Array = Uint8Array.from(privateKeyArray);

    // Create a Keypair from the private key
    const keypair = Keypair.fromSecretKey(privateKeyUint8Array);

    // Encode the private key to a Base58 string
    const base58String = bs58.encode(keypair.secretKey);

    console.log('Base58 Encoded String:', base58String);

    // Decode the Base58 string back to a Uint8Array
    //const decodedPrivateKeyUint8Array = bs58.decode(base58String);
    // const decodedPrivateKeyUint8Array = bs58.decode('');

    // Convert the Uint8Array back to an array of integers
    // const decodedPrivateKeyArray = Array.from(decodedPrivateKeyUint8Array);

    // console.log('Decoded Private Key Array:', decodedPrivateKeyArray);
  });
});
