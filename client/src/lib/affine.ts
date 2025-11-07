const CHARSET =
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

function gcd(a: number, b: number): number {
  while (b != 0) {
    [a, b] = [b, a % b];
  }
  return a;
}

function encryptAffineChar(a: number, b: number, char: string) {
  const index = CHARSET.indexOf(char);

  if (index == -1) return char;

  const newIndex = (a * index + b) % CHARSET.length;
  return CHARSET.at(newIndex);
}

export function encryptAffine(a: number, b: number, plain: string) {
  if (gcd(a, CHARSET.length) != 1)
    throw new Error(`A harus koprima dengan ${CHARSET.length}`);

  return plain
    .split("")
    .map((char) => encryptAffineChar(a, b, char))
    .join("");
}

function getInvers(a: number): number {
  const m = CHARSET.length;
  for (let x = 1; x < m; x++) {
    if ((a * x) % m == 1) {
      return x;
    }
  }
  return -1;
}

function decryptAffineChar(aInvers: number, b: number, char: string) {
  const index = CHARSET.indexOf(char);

  if (index == -1) return char;

  const mod = CHARSET.length;
  const newIndex = (aInvers * ((index - b + mod) % mod)) % mod;
  return CHARSET.at(newIndex);
}

export function decryptAffine(a: number, b: number, cipher: string) {
  const aInvers = getInvers(a);
  if (aInvers == -1)
    throw new Error(
      `Kunci A tidak valid. A harus koprima dengan ${CHARSET.length}`
    );

  return cipher
    .split("")
    .map((char) => decryptAffineChar(aInvers, b, char))
    .join("");
}
