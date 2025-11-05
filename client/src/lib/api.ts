import { createSRPClient } from "@levirs565/srp";
import blake from "blakejs";
import axios, { type AxiosResponse, type AxiosResponseHeaders } from "axios";
import { fromHex, toHex } from "@smithy/util-hex-encoding";
import { fromBase64, toBase64 } from "@smithy/util-base64";
import {
  encryptAESGCM,
  encryptAESGCMPassword,
  encryptRSA,
  exportRawKey,
  exportRSAPrivateKey,
  exportRSAPublicKey,
  generateRSAKeyPair,
  importRSAPublicKey,
  generateAESGCMKey,
  typedArrayToBuffer,
  importRSAPrivateKey,
  decryptAESGCMPassword,
  decryptRSA,
  importAESGCMKey,
  decryptAESGCM,
} from "./crypto";
import { getUserPrivateKey, putUserPrivateKey } from "./db";

const srpClient = createSRPClient("SHA-256", 2048);

const API_URL = "http://localhost:8080";
const instance = axios.create({
  baseURL: API_URL,
  withCredentials: true,
});

async function runRequest(runner: () => Promise<AxiosResponse>) {
  try {
    const response = await runner();

    if (response.data.error) {
      throw response.data.error;
    }

    return response.data;
  } catch (e: any) {
    if (axios.isAxiosError(e)) {
      if (e.response) {
        throw e.response.data.error;
      } else {
        throw { type: "NetworkError", error: e };
      }
    } else {
      throw { type: "Unknown", error: e };
    }
  }
}

const get = (path: string) => runRequest(() => instance.get(path));
const post = (path: string, data: any) =>
  runRequest(() => instance.post(path, data));

async function signup(username: string, password: string) {
  const keyPair = await generateRSAKeyPair();
  const privateKey = await exportRSAPrivateKey(keyPair.privateKey);
  const publicKey = await exportRSAPublicKey(keyPair.publicKey);

  const privateKeyEncrypted = await encryptAESGCMPassword(password, privateKey);

  const srpSalt = srpClient.generateSalt();
  const srpPrivateKey = await srpClient.derivePrivateKey(
    srpSalt,
    username,
    password
  );
  const srpVerifier = await srpClient.deriveVerifier(srpPrivateKey);

  return instance.post("/auth/signup", {
    username,
    srp_salt: toBase64(fromHex(srpSalt)),
    srp_verifier: toBase64(fromHex(srpVerifier)),
    private_key_encrypted: toBase64(new Uint8Array(privateKeyEncrypted)),
    public_key: toBase64(new Uint8Array(publicKey)),
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
  );

  const rsaPrivateKeyRawEncrypted = fromBase64(
    (await getKeys()).private_key_encrypted
  );
  const rsaPrivateKeyRaw = await decryptAESGCMPassword(
    password,
    rsaPrivateKeyRawEncrypted
  );
  const rsaPrivateKey = await importRSAPrivateKey(rsaPrivateKeyRaw);

  await putUserPrivateKey(rsaPrivateKey);

  return true;
}

async function logout() {
  return post("/auth/logout", null);
}

async function getState() {
  return get("/auth/state");
}

async function getKeys() {
  return get("/auth/keys");
}

async function getUser(id: string) {
  return get(`/user/${id}`);
}

async function encryptKeyForUser(userid: string, keyBuffer: ArrayBuffer) {
  const publicKeyRaw = typedArrayToBuffer(
    fromBase64((await getUser(userid)).public_key)
  );
  return encryptRSA(await importRSAPublicKey(publicKeyRaw), keyBuffer);
}

async function getUserId() {
  return (await getState()).user.id;
}

async function getChatData(receiverId: string, message: string) {
  const messageKey = await generateAESGCMKey();
  const keyBuffer = await exportRawKey(messageKey);
  return {
    receiver_id: receiverId,
    message_cipher: toBase64(
      await encryptAESGCM(messageKey, new TextEncoder().encode(message))
    ),
    receiver_key: toBase64(
      new Uint8Array(await encryptKeyForUser(receiverId, keyBuffer))
    ),
    sender_key: toBase64(
      new Uint8Array(await encryptKeyForUser(await getUserId(), keyBuffer))
    ),
  };
}

async function decryptChat(
  user_id: string,
  { sender_key, receiver_key, cipher, ...other }: any
) {
  const encryptedKey = fromBase64(
    other.sender_id == user_id ? sender_key : receiver_key
  );
  const keyRaw = await decryptRSA(
    await getUserPrivateKey(),
    new Uint8Array(encryptedKey)
  );
  const key = await importAESGCMKey(keyRaw);
  return {
    ...other,
    message: new TextDecoder().decode(
      await decryptAESGCM(key, fromBase64(cipher))
    ),
  };
}

async function getChats() {
  const userPrivateKey = await getUserPrivateKey();
  const id = (await getState()).user.id;
  return Promise.all((await get("/chat")).map(decryptChat));
}

async function runChatWs() {
   const id = (await getState()).user.id;

  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`${API_URL}/chat/ws`);
    const errorListener = () => {
      ws.removeEventListener("error", errorListener);
      reject();
    };
    ws.addEventListener("error", errorListener);
    ws.addEventListener("open", () => {
      const chatListeners: any[] = [];

      ws.addEventListener("message", async (e) => {
        const message = e.data;
        if (typeof message == "string") {
          try {
            const { type, ...rest } = JSON.parse(message);
            if (type == "ReceiveChat") {
              const chat = await decryptChat(id, rest);
              for (const listener of chatListeners) {
                listener(chat);
              }
            }
          } catch {
            console.log(`Server: ${message}`);
          }
        }
      });

      resolve({
        send: async (receiverId: string, message: string) => {
          ws.send(
            JSON.stringify({
              type: "SendChat",
              ...(await getChatData(receiverId, message)),
            })
          );
        },
        addChatListener: (listener: any) => {
          chatListeners.push(listener);
        },
      });
    });
  });
}

async function runChatWsTest() {
  const ws = await runChatWs();
  ws.addChatListener((chat) => {
    console.log(chat.message);
  })
  return ws
}

(globalThis as any).API = {
  signup,
  login,
  logout,
  getState,
  getKeys,
  getUser,
  getChats,
  runChatWs: runChatWsTest,
};
