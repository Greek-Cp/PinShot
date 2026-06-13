// One-off generator for a placeholder source icon (1024×1024 PNG).
// Run: node scripts/gen-icon.mjs  →  app-icon.png
// Then: npx @tauri-apps/cli@2 icon app-icon.png -o src-tauri/icons
import { deflateSync } from 'node:zlib';
import { writeFileSync } from 'node:fs';

const SIZE = 1024;

// CRC32 (PNG chunk checksum).
const crcTable = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();
function crc32(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = crcTable[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, 'ascii');
  const body = Buffer.concat([typeBuf, data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body), 0);
  return Buffer.concat([len, body, crc]);
}

// Pixel art: indigo background, white rounded square with a "pin" dot.
const bg = [79, 70, 229, 255]; // indigo
const fg = [255, 255, 255, 255];
const raw = Buffer.alloc(SIZE * (1 + SIZE * 4));
const cornerR = SIZE * 0.16;
const m = SIZE * 0.2; // margin of the inner square
const lo = m;
const hi = SIZE - m;
function insideRounded(x, y) {
  if (x < lo || x > hi || y < lo || y > hi) return false;
  const ix = Math.min(Math.max(x, lo + cornerR), hi - cornerR);
  const iy = Math.min(Math.max(y, lo + cornerR), hi - cornerR);
  const dx = x - ix;
  const dy = y - iy;
  return dx * dx + dy * dy <= cornerR * cornerR;
}
for (let y = 0; y < SIZE; y++) {
  const rowStart = y * (1 + SIZE * 4);
  raw[rowStart] = 0; // filter: none
  for (let x = 0; x < SIZE; x++) {
    const px = insideRounded(x, y) ? fg : bg;
    const o = rowStart + 1 + x * 4;
    raw[o] = px[0];
    raw[o + 1] = px[1];
    raw[o + 2] = px[2];
    raw[o + 3] = px[3];
  }
}

const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // color type RGBA
const png = Buffer.concat([
  sig,
  chunk('IHDR', ihdr),
  chunk('IDAT', deflateSync(raw, { level: 9 })),
  chunk('IEND', Buffer.alloc(0)),
]);
writeFileSync('app-icon.png', png);
console.log('wrote app-icon.png', png.length, 'bytes');
