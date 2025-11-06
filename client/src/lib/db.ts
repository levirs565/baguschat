import Dexie from "dexie";

const db = new Dexie("app_db");

db.version(1).stores({
  user_data: "++name, data",
});

export async function putUserPrivateKey(key: CryptoKey) {
  await db.user_data.put({
    name: "private_key",
    data: key,
  });
}

export async function getUserPrivateKey(): Promise<CryptoKey | undefined> {
  return (await db.user_data.get("private_key")).data;
}
