import { randomBytes } from "@noble/ciphers/utils.js";
import { typedArrayToBuffer } from "./crypto";
import { fromHex, toHex } from "@smithy/util-hex-encoding";

const MARKER_START = new TextEncoder().encode("::PESAN_RAHASIA::")
const MARKER_END = fromHex("00FFAA55");

const MARKER = new Uint8Array(MARKER_START.byteLength + MARKER_END.byteLength);

MARKER.set(MARKER_START, 0)
MARKER.set(MARKER_END, MARKER_START.byteLength)

export function addEOFMessage(blob: ArrayBuffer, message: string) {
  const encoded = new TextEncoder().encode(message);
  const result = new Uint8Array(
    blob.byteLength + MARKER.length + encoded.length
  );
  result.set(new Uint8Array(blob), 0);
  result.set(MARKER, blob.byteLength);
  result.set(encoded, blob.byteLength + MARKER.length);
  return result;
}

function isEqual(a: Uint8Array, b: Uint8Array) {
  if (a.byteLength != b.byteLength) return false;

  for (let i = 0; i < a.byteLength; i++) {
    if (a.at(i) != b.at(i)) return false;
  }

  return true;
}

export function extractEOFMessage(blob: Uint8Array) {
  if (blob.byteLength < MARKER.byteLength) return null;

  for (let i = blob.byteLength - MARKER.byteLength; i >= 0; i--) {
    const subarray = blob.subarray(i, i + MARKER.byteLength);
    if (isEqual(subarray, MARKER)) {
      return new TextDecoder().decode(blob.subarray(i + MARKER.byteLength));
    }
  }
  return null;
}