import { xchacha20poly1305 } from "@noble/ciphers/chacha.js"
import { randomBytes } from "@noble/ciphers/utils.js"

export function typedArrayToBuffer(array: Uint8Array): ArrayBuffer {
  return array.buffer.slice(
    array.byteOffset,
    array.byteLength + array.byteOffset
  ) as ArrayBuffer;
}

export async function generateAESGCMKey() {
  return crypto.subtle.generateKey(
    {
      name: "AES-GCM",
      length: 256,
    },
    true,
    ["encrypt", "decrypt"]
  );
}

export async function importAESGCMKey(buffer: BufferSource) {
  return crypto.subtle.importKey("raw", buffer, { name: "AES-GCM" }, false, [
    "encrypt",
    "decrypt",
  ]);
}

export async function encryptAESGCM(key: CryptoKey, buffer: BufferSource) {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encrypted = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv: iv },
    key,
    buffer
  );

  const result = new Uint8Array(iv.length + encrypted.byteLength);
  result.set(iv, 0);
  result.set(new Uint8Array(encrypted), iv.length);
  return result;
}

export async function decryptAESGCM(key: CryptoKey, buffer: Uint8Array) {
  const iv = typedArrayToBuffer(buffer.subarray(0, 12));
  const cipher = typedArrayToBuffer(buffer.subarray(12));

  return crypto.subtle.decrypt({ name: "AES-GCM", iv: iv }, key, cipher);
}

export async function deriveAESGCMKey(salt: Uint8Array, password: string) {
  const key = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(password),
    "PBKDF2",
    false,
    ["deriveKey"]
  );

  return await crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: typedArrayToBuffer(salt),
      iterations: 100000,
      hash: "SHA-256",
    },
    key,
    { name: "AES-GCM", length: 256 },
    false,
    ["encrypt", "decrypt"]
  );
}

export async function encryptAESGCMPassword(
  password: string,
  buffer: BufferSource
) {
  const salt = crypto.getRandomValues(new Uint8Array(16));

  const aesKey = await deriveAESGCMKey(salt, password);
  const encrypted = await encryptAESGCM(aesKey, buffer);

  const result = new Uint8Array(salt.length + encrypted.byteLength);
  result.set(salt, 0);
  result.set(new Uint8Array(encrypted), salt.length);
  return typedArrayToBuffer(result);
}

export async function decryptAESGCMPassword(
  password: string,
  buffer: Uint8Array
) {
  const salt = buffer.subarray(0, 16);
  const cipher = buffer.subarray(16);
  const aesKey = await deriveAESGCMKey(salt, password);

  return decryptAESGCM(aesKey, cipher);
}

export async function exportRawKey(key: CryptoKey) {
  return crypto.subtle.exportKey("raw", key);
}

export async function generateRSAKeyPair() {
  return crypto.subtle.generateKey(
    {
      name: "RSA-OAEP",
      modulusLength: 2048,
      publicExponent: new Uint8Array([1, 0, 1]),
      hash: "SHA-256",
    },
    true,
    ["encrypt", "decrypt"]
  );
}

export async function encryptRSA(publicKey: CryptoKey, buffer: BufferSource) {
  return await crypto.subtle.encrypt(
    {
      name: "RSA-OAEP",
    },
    publicKey,
    buffer
  );
}

export async function decryptRSA(privateKey: CryptoKey, buffer: BufferSource) {
  return await crypto.subtle.decrypt(
    {
      name: "RSA-OAEP",
    },
    privateKey,
    buffer
  );
}

export async function importRSAPrivateKey(buffer: BufferSource) {
  return await crypto.subtle.importKey(
    "pkcs8",
    buffer,
    {
      name: "RSA-OAEP",
      hash: "SHA-256",
    },
    false,
    ["decrypt"]
  );
}

export async function importRSAPublicKey(buffer: BufferSource) {
  return await crypto.subtle.importKey(
    "spki",
    buffer,
    {
      name: "RSA-OAEP",
      hash: "SHA-256",
    },
    false,
    ["encrypt"]
  );
}

export async function exportRSAPrivateKey(key: CryptoKey) {
  return crypto.subtle.exportKey("pkcs8", key);
}

export async function exportRSAPublicKey(key: CryptoKey) {
  return crypto.subtle.exportKey("spki", key);
}

export function generateXcacha20Key() {
  return randomBytes(32);
}

export function encryptXcacha20(key: Uint8Array<ArrayBufferLike>, buffer: Uint8Array<ArrayBufferLike>) {
  const nonce = randomBytes(24);
  const cacha = xchacha20poly1305(key, nonce);
  const encrypted = cacha.encrypt(buffer);
  const result = new Uint8Array(nonce.byteLength + encrypted.byteLength);
  result.set(nonce, 0);
  result.set(encrypted, nonce.length);
  return result
}

export function decryptXcacha20(key: Uint8Array<ArrayBufferLike>, buffer: Uint8Array<ArrayBufferLike>) {
  const nonce = buffer.subarray(0, 24);
  const cipher = buffer.subarray(24);
   const cacha = xchacha20poly1305(key, nonce);
   return cacha.decrypt(cipher);
}

(globalThis as any).Crypto = {
  generateAESGCMKey,
  encryptAESGCM,
  decryptAESGCM,
  encryptAESGCMPassword,
  decryptAESGCMPassword,
};
