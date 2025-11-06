import { createSRPClient } from "@levirs565/srp";
import blake from "blakejs";
import axios, {
  type AxiosInstance,
  type AxiosResponse,
  type AxiosResponseHeaders,
} from "axios";
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
import { TypedEventTarget } from "typescript-event-target";

const srpClient = createSRPClient("SHA-256", 2048);

interface SignupRequest {
  username: string;
  srp_salt: string;
  srp_verifier: string;
  private_key_encrypted: string;
  public_key: string;
}

interface SRPHelloRequest {
  username: string;
  srp_client_public_key: string;
}

interface SRPHelloResponse {
  srp_salt: string;
  srp_server_public_key: string;
}

interface ActionResultResponse {
  success: boolean;
}

interface SRPAuthRequest {
  srp_evidence: string;
}

interface SRPAuthResponse {
  srp_evidence: string;
}

interface GetStateResponseUser {
  id: string;
  username: String;
}

interface GetStateResponse {
  user?: GetStateResponseUser;
}

interface GetKeysResponse {
  private_key_encrypted: string;
  public_key: string;
}

interface GetUserResponse {
  username: string;
  public_key: string;
}

type ChatItem = {
  id: string;
  created_at: string;
  sender_id: string;
  receiver_id?: string;
  sender_key: string;
  receiver_key: string;
} & (
  | {
      type: "Text";
      cipher: string;
    }
  | {
      type: "File";
      file_type: "File" | "Image";
      filename: string;
      mime_type: string;
      size: number;
      path: string;
    }
);

interface ChatPartners {
  id: string;
  last_chat: ChatItem;
}

class APIService {
  instance: AxiosInstance;
  baseUrl: string;
  constructor(apiUrl: string = "http://localhost:8080") {
    this.baseUrl = apiUrl;
    this.instance = axios.create({
      baseURL: apiUrl,
      withCredentials: true,
    });
  }

  async runRequest<T>(runner: () => Promise<AxiosResponse>): Promise<T> {
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

  get<T>(path: string): Promise<T> {
    return this.runRequest(() => this.instance.get(path));
  }
  post<T>(path: string, data: any): Promise<T> {
    return this.runRequest(() => this.instance.post(path, data));
  }

  signup(data: SignupRequest) {
    return this.post<ActionResultResponse>("/auth/signup", data);
  }

  srpHello(data: SRPHelloRequest) {
    return this.post<SRPHelloResponse>("/auth/hello", data);
  }

  srpAuth(data: SRPAuthRequest) {
    return this.post<SRPAuthResponse>("/auth/auth", data);
  }

  logout() {
    return this.post<ActionResultResponse>("/auth/logout", null);
  }

  getState() {
    return this.get<GetStateResponse>("/auth/state");
  }

  getKeys() {
    return this.get<GetKeysResponse>("/auth/keys");
  }

  getUser(id: string) {
    return this.get<GetUserResponse>(`/user/${id}`);
  }

  getChats() {
    return this.get<ChatItem[]>(`/chat`);
  }

  getChatPartners() {
    return this.get<ChatPartners[]>(`/chat/partners`);
  }
}

class ClientService {
  #apiService: APIService;

  constructor(apiService: APIService) {
    this.#apiService = apiService;
  }

  async signup(username: string, password: string) {
    const keyPair = await generateRSAKeyPair();
    const privateKey = await exportRSAPrivateKey(keyPair.privateKey);
    const publicKey = await exportRSAPublicKey(keyPair.publicKey);

    const privateKeyEncrypted = await encryptAESGCMPassword(
      password,
      privateKey
    );

    const srpSalt = srpClient.generateSalt();
    const srpPrivateKey = await srpClient.derivePrivateKey(
      srpSalt,
      username,
      password
    );
    const srpVerifier = await srpClient.deriveVerifier(srpPrivateKey);

    return api.signup({
      username,
      srp_salt: toBase64(fromHex(srpSalt)),
      srp_verifier: toBase64(fromHex(srpVerifier)),
      private_key_encrypted: toBase64(new Uint8Array(privateKeyEncrypted)),
      public_key: toBase64(new Uint8Array(publicKey)),
    });
  }

  async login(username: string, password: string) {
    const clientKey = srpClient.generateEphemeral();

    const halloResponse = await api.srpHello({
      username,
      srp_client_public_key: toBase64(fromHex(clientKey.public)),
    });

    const salt = toHex(fromBase64(halloResponse.srp_salt));
    const serverPublicKey = toHex(
      fromBase64(halloResponse.srp_server_public_key as string)
    );

    const privateKey = await srpClient.derivePrivateKey(
      salt,
      username,
      password
    );
    const clientSession = await srpClient.deriveSession(
      clientKey.secret,
      serverPublicKey,
      salt,
      username,
      privateKey
    );

    const authResponse = await api.srpAuth({
      srp_evidence: toBase64(fromHex(clientSession.proof)),
    });

    await srpClient.verifySession(
      clientKey.public,
      clientSession,
      toHex(fromBase64(authResponse.srp_evidence))
    );

    return this.createClientSession(password);
  }

  async createClientSession(password: string | undefined) {
    const userKeys = await api.getKeys();

    if (password) {
      const rsaPrivateKeyRawEncrypted = fromBase64(
        userKeys.private_key_encrypted
      );
      const rsaPrivateKeyRaw = await decryptAESGCMPassword(
        password,
        rsaPrivateKeyRawEncrypted
      );
      const rsaPrivateKey = await importRSAPrivateKey(rsaPrivateKeyRaw);

      await putUserPrivateKey(rsaPrivateKey);
    }

    const privateKey = await getUserPrivateKey();

    if (!privateKey) return null;

    const publicKeyRaw = fromBase64(userKeys.public_key);
    const publicKey = await importRSAPublicKey(new Uint8Array(publicKeyRaw));

    const userState = await api.getState();

    if (!userState.user) return null;

    return new ClientSession(
      this.#apiService,
      userState.user.id,
      privateKey,
      publicKey
    );
  }
}

class ClientSession {
  #cryptoService: CryptoService;
  #apiService: APIService;

  constructor(
    apiService: APIService,
    userid: string,
    privateKey: CryptoKey,
    publickey: CryptoKey
  ) {
    this.#apiService = apiService;
    this.#cryptoService = new CryptoService(
      apiService,
      userid,
      privateKey,
      publickey
    );
  }

  async getChats() {
    return Promise.all(
      (await api.getChats()).map(
        this.#cryptoService.decryptChat2.bind(this.#cryptoService)
      )
    );
  }

  async getChatPartners() {
    return Promise.all(
      (await api.getChatPartners()).map(async ({ last_chat, ...other }) => ({
        ...other,
        last_chat: await this.#cryptoService.decryptChat2(last_chat),
      }))
    );
  }

  async createChatWs(): Promise<ChatWs> {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket(`${this.#apiService.baseUrl}/chat/ws`);

      const errorListener = () => {
        ws.removeEventListener("error", errorListener);
        reject();
      };

      ws.addEventListener("error", errorListener);

      ws.addEventListener("open", () => {
        resolve(new ChatWs(ws, this.#cryptoService));
      });
    });
  }
}

class CryptoService {
  #apiService: APIService;

  #currentUserId: string;
  #currentPrivateKey: CryptoKey;
  #currentPublicKey: CryptoKey;

  #cachedPublicKey: Map<string, CryptoKey>;

  constructor(
    apiService: APIService,
    currentUserId: string,
    privateKey: CryptoKey,
    publicKey: CryptoKey
  ) {
    this.#apiService = apiService;
    this.#currentUserId = currentUserId;
    this.#currentPrivateKey = privateKey;
    this.#currentPublicKey = publicKey;
    this.#cachedPublicKey = new Map();
  }

  getCurrentPrivateKey() {
    return this.#currentPrivateKey;
  }

  getCurrentPublicKey() {
    return this.#currentPublicKey;
  }

  async getPublicKey(userId: string): Promise<CryptoKey> {
    if (userId == this.#currentUserId) {
      return this.#currentPublicKey;
    }

    if (this.#cachedPublicKey.has(userId)) {
      return this.#cachedPublicKey.get(userId)!;
    }

    const raw = typedArrayToBuffer(
      fromBase64((await this.#apiService.getUser(userId)).public_key)
    );
    const key = await importRSAPublicKey(raw);

    this.#cachedPublicKey.set(userId, key);

    return key;
  }

  async encrypt(userId: string, buffer: BufferSource) {
    return encryptRSA(await this.getPublicKey(userId), buffer);
  }

  async getChatData(receiverId: string, message: string) {
    const messageKey = await generateAESGCMKey();
    const keyBuffer = await exportRawKey(messageKey);
    return {
      receiver_id: receiverId,
      message_cipher: toBase64(
        await encryptAESGCM(messageKey, new TextEncoder().encode(message))
      ),
      receiver_key: toBase64(
        new Uint8Array(await this.encrypt(receiverId, keyBuffer))
      ),
      sender_key: toBase64(
        new Uint8Array(await this.encrypt(this.#currentUserId, keyBuffer))
      ),
    };
  }

  async decryptChat2({ sender_key, receiver_key, ...other }: ChatItem) {
    const encryptedKey = fromBase64(
      other.sender_id == this.#currentUserId ? sender_key : receiver_key
    );
    const keyRaw = await decryptRSA(
      this.#currentPrivateKey,
      new Uint8Array(encryptedKey)
    );
    const key = await importAESGCMKey(keyRaw);
    const base = {
      id: other.id,
      created_at: other.created_at,
      sender_id: other.sender_id,
      receiver_id: other.receiver_id,
    };

    if (other.type == "Text")
      return {
        ...base,
        type: other.type,
        message: new TextDecoder().decode(
          await decryptAESGCM(key, fromBase64(other.cipher))
        ),
      };
    else
      return {
        ...base,
        type: other.type,
        key,
        file_type: other.file_type,
        filename: other.filename,
        mime_type: other.mime_type,
        size: other.size,
        path: other.path,
      };
  }
}

interface SendChatRequest {
  receiver_id: string;
  sender_key: string;
  receiver_key: string;
  message_cipher: string;
}

type WsRequest = { type: "SendChat" } & SendChatRequest;
type WsReponseMessage = { type: "ReceiveChat"; chat: ChatItem };

type DecryptedChat2 = Awaited<ReturnType<CryptoService["decryptChat2"]>>;

interface ChatWsEventMap {
  "receive-chat": CustomEvent<DecryptedChat2>;
}

class ChatWs extends TypedEventTarget<ChatWsEventMap> {
  ws: WebSocket;
  #cryptoservice: CryptoService;

  constructor(ws: WebSocket, cryptoService: CryptoService) {
    super();
    this.ws = ws;
    this.#cryptoservice = cryptoService;

    this.ws.addEventListener("message", this.#onWsMessage.bind(this));
  }

  sendWsRequest(request: WsRequest) {
    this.ws.send(JSON.stringify(request));
  }

  async sendText(receiverId: string, message: string) {
    this.sendWsRequest({
      type: "SendChat",
      ...(await this.#cryptoservice.getChatData(receiverId, message)),
    });
  }

  async #onWsMessage(e: WebSocketEventMap["message"]) {
    const message = e.data;
    if (typeof message == "string") {
      try {
        const { type, ...rest } = JSON.parse(message) as WsReponseMessage;
        if (type == "ReceiveChat") {
          const chat = await this.#cryptoservice.decryptChat2(rest.chat);
          this.dispatchTypedEvent(
            "receive-chat",
            new CustomEvent("receive-chat", {
              detail: chat,
            })
          );
        }
      } catch (e) {
        console.log(`Server: ${message}`);
      }
    }
  }
}

async function runChatWsTest() {
  const ws = (await (
    await client.createClientSession(undefined)
  )?.createChatWs())!;
  ws.addEventListener("receive-chat", (e) => {
    if (e.detail.type == "Text") console.log(e.detail.message);
  });
  return ws;
}

export const api = new APIService();
export const client = new ClientService(api);

(globalThis as any).API = {
  api,
  client,
  runChatWs: runChatWsTest,
};
