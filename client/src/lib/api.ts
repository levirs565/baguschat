import { createSRPClient } from "@levirs565/srp";
import blake from "blakejs";
import axios, { type AxiosResponseHeaders } from "axios";
import { fromHex, toHex } from "@smithy/util-hex-encoding";
import { fromBase64, toBase64 } from "@smithy/util-base64";

function typedArrayToBuffer(array: Uint8Array): ArrayBuffer {
  return array.buffer.slice(
    array.byteOffset,
    array.byteLength + array.byteOffset
  ) as ArrayBuffer;
}

const srpClient = createSRPClient(
  "SHA-256",
  2048
);

const API_URL = "http://localhost:8080";
const instance = axios.create({
  baseURL: API_URL,
  withCredentials: true
});


async function post(path: string, data: any) {
  try {
    const response = await instance.post(path, data);

    if (response.data.error) {
      throw response.data.error;
    }

    return response.data;
  } catch (e: any) {
    if (axios.isAxiosError(e)) {
      if (e.response) {
        throw e.response.data.error
      } else {
        throw {type:"NetworkError", error: e}
      }
    } else {
        throw {type:"Unknown", error: e}
    }
  }
}

async function signup(username: string, password: string) {
  const keyPair = await crypto.subtle.generateKey(
    {
      name: "RSA-OAEP",
      modulusLength: 2048,
      publicExponent: new Uint8Array([1, 0, 1]),
      hash: "SHA-256",
    },
    true,
    ["encrypt", "decrypt"]
  );

  const pkcs8 = await crypto.subtle.exportKey("pkcs8", keyPair.privateKey);
  const spki = await crypto.subtle.exportKey("spki", keyPair.publicKey);
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const iv = crypto.getRandomValues(new Uint8Array(12));

  const passwordKey = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(password),
    "PBKDF2",
    false,
    ["deriveKey"]
  );

  const aesKey = await crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: salt,
      iterations: 100000,
      hash: "SHA-256",
    },
    passwordKey,
    { name: "AES-GCM", length: 256 },
    false,
    ["encrypt"]
  );

  const privateKeyEncrypted = await crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv: iv,
    },
    aesKey,
    pkcs8
  );

  const srpSalt = srpClient.generateSalt();
  const srpPrivateKey = await srpClient.derivePrivateKey(
    srpSalt,
    username,
    password
  );
  const srpVerifier = await srpClient.deriveVerifier(srpPrivateKey);
  console.log(srpSalt)

  return instance.post("/auth/signup", {
    username,
    srp_salt: toBase64(fromHex(srpSalt)),
    srp_verifier: toBase64(fromHex(srpVerifier)),
    private_key_encrypted: toBase64(new Uint8Array(privateKeyEncrypted)),
    public_key: toBase64(new Uint8Array(spki)),
  });
}

async function login(username: string, password: string) {
  const clientKey = srpClient.generateEphemeral();

  const halloResponse = await post("/auth/hello", {
    username,
    srp_client_public_key: toBase64(fromHex(clientKey.public)),
  });

  const salt = toHex(fromBase64(halloResponse.srp_salt));
  const serverPublicKey = toHex(
    fromBase64(halloResponse.srp_server_public_key as string)
  );

  const privateKey = await srpClient.derivePrivateKey(salt, username, password);
  const clientSession = await srpClient.deriveSession(
    clientKey.secret,
    serverPublicKey,
    salt,
    username,
    privateKey
  );
  
  const authResponse = await post("/auth/auth", {
    srp_evidence: toBase64(fromHex(clientSession.proof)),
  });

  await srpClient.verifySession(
    clientKey.public,
    clientSession,
    toHex(fromBase64(authResponse.srp_evidence))
  )
  return true
}

async function logout() {
  return post("/auth/logout", null);
}

globalThis.API = {
  signup,
  login,
  logout
}
