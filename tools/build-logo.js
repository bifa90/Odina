#!/usr/bin/env node
/**
 * Odina logo build — generates every shipped asset from the vector sources.
 * No external deps besides `sharp`. Run: node tools/build-logo.js
 *
 * Design ref: docs/design/logo-spec-v1.md  (also §36 of the agent prompt)
 */
const fs = require('fs');
const path = require('path');
const sharp = require('sharp');

const NAVY = '#0A2540';
const GOLD = '#D4AF37';
const SAND = '#E8D5A9';
const WHITE = '#FFFFFF';

const OUT = 'assets/logo';

/* ── geometry ────────────────────────────────────────────────────────────
   Pointed Islamic arch (Bab Musa, Taiz) closed by a circular base so the
   whole silhouette also reads as a stylised "O". */
const ARCH = 'M 512 176 C 622 214 788 320 788 500 L 788 688 ' +
             'C 788 794 664 852 512 852 C 360 852 236 794 236 688 ' +
             'L 236 500 C 236 320 402 214 512 176 Z';

/* Inner pillar = the guardian / the passage. Rounded top, never pointed. */
const PILLAR = 'M 476 592 Q 476 566 502 566 L 522 566 Q 548 566 548 592 ' +
               'L 548 756 L 476 756 Z';

/* Small-size variant (<= 32px): no pillar, heavier stroke, blunter profile. */
const ARCH_SMALL = 'M 512 262 C 596 306 696 384 696 508 L 696 690 ' +
                   'C 696 758 612 790 512 790 C 412 790 328 758 328 690 ' +
                   'L 328 508 C 328 384 428 306 512 262 Z';

function svgMarkup({ stroke, accent, strokeWidth = 112, pillar = true, bg = null }) {
  const bgRect = bg ? `<rect width="1024" height="1024" fill="${bg}"/>` : '';
  const arch = `<path d="${ARCH}" fill="none" stroke="${stroke}" stroke-width="${strokeWidth}" ` +
               `stroke-linejoin="round" stroke-linecap="round"/>`;
  const pil = pillar ? `<path d="${PILLAR}" fill="${accent}"/>` : '';
  /* SCALE: content fills ~88% of the canvas (12% safe margin) like the
     reference density — same 0.83 portrait ratio, larger optical weight. */
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">` +
         `${bgRect}<g transform="translate(512 512) scale(1.13) translate(-512 -512)">` +
         `${arch}${pil}</g></svg>\n`;
}

function svgSmall(color) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">` +
         `<g transform="translate(512 512) scale(1.13) translate(-512 -512)">` +
         `<path d="${ARCH_SMALL}" fill="none" stroke="${color}" stroke-width="150" ` +
         `stroke-linejoin="round" stroke-linecap="round"/></g></svg>\n`;
}

const VARIANTS = {
  'light':        () => svgMarkup({ stroke: NAVY, accent: GOLD }),
  'dark':         () => svgMarkup({ stroke: GOLD, accent: GOLD, bg: NAVY }),
  'mono-black':   () => svgMarkup({ stroke: '#000000', accent: '#000000' }),
  'mono-white':   () => svgMarkup({ stroke: WHITE, accent: WHITE }),
};
const SMALL_VARIANTS = {
  'light': '#0A2540',
  'dark':  '#D4AF37',
  'mono-black': '#000000',
  'mono-white': '#FFFFFF',
};

const SIZES = [1024, 512, 256, 128, 64, 48, 32, 24, 16];
const SIZES_SMALL = new Set([32, 24, 16]);

function mkdirp(p) { fs.mkdirSync(p, { recursive: true }); }

async function renderPng(svgStr, size, flattenBg) {
  let img = sharp(Buffer.from(svgStr)).resize(size, size, { fit: 'contain' });
  if (flattenBg) img = img.flatten({ background: flattenBg });
  return img.png({ compressionLevel: 9, palette: false }).toBuffer();
}

/* ICO container: PNG-compressed entries (valid for Vista+). */
function buildIco(frames) {
  const count = frames.length;
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); header.writeUInt16LE(1, 2); header.writeUInt16LE(count, 4);
  const dir = Buffer.alloc(16 * count);
  let offset = 6 + 16 * count;
  const bodies = [];
  frames.forEach((f, i) => {
    const b = 16 * i;
    dir.writeUInt8(f.size >= 256 ? 0 : f.size, b + 0);
    dir.writeUInt8(f.size >= 256 ? 0 : f.size, b + 1);
    dir.writeUInt8(0, b + 2); dir.writeUInt8(0, b + 3);
    dir.writeUInt16LE(1, b + 4); dir.writeUInt16LE(32, b + 6);
    dir.writeUInt32LE(f.data.length, b + 8);
    dir.writeUInt32LE(offset, b + 12);
    offset += f.data.length;
    bodies.push(f.data);
  });
  return Buffer.concat([header, dir, ...bodies]);
}

/* Sand-texture hero background (procedural noise, no external image). */
async function sandTexture(size) {
  const px = Buffer.alloc(size * size * 3);
  let seed = 1337;
  const rnd = () => (seed = (seed * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff;
  const base = { r: 0xE8, g: 0xD5, b: 0xA9 };
  for (let i = 0; i < size * size; i++) {
    const n = (rnd() - 0.5) * 14;
    px[i * 3 + 0] = Math.max(0, Math.min(255, base.r + n));
    px[i * 3 + 1] = Math.max(0, Math.min(255, base.g + n * 0.9));
    px[i * 3 + 2] = Math.max(0, Math.min(255, base.b + n * 0.8));
  }
  return sharp(px, { raw: { width: size, height: size, channels: 3 } }).png().toBuffer();
}

(async () => {
  mkdirp(`${OUT}/svg`);
  for (const v of Object.keys(VARIANTS)) {
    mkdirp(`${OUT}/png/${v}`);
    // vector sources
    fs.writeFileSync(`${OUT}/svg/odina-icon-${v}.svg`, VARIANTS[v]());
    // PNG ladder
    for (const size of SIZES) {
      const src = SIZES_SMALL.has(size) ? svgSmall(SMALL_VARIANTS[v]) : VARIANTS[v]();
      const flat = v === 'dark' ? NAVY : (v === 'mono-white' ? NAVY : null);
      const buf = await renderPng(src, size, flat);
      fs.writeFileSync(`${OUT}/png/${v}/odina-${size}.png`, buf);
    }
  }
  // dedicated small-size vector
  fs.writeFileSync(`${OUT}/svg/odina-icon-16-light.svg`, svgSmall(NAVY));

  // Windows .ico (from light variant, transparent)
  const icoFrames = [];
  for (const size of [16, 24, 32, 48, 64, 128, 256]) {
    const src = SIZES_SMALL.has(size) ? svgSmall(NAVY) : VARIANTS['light']();
    icoFrames.push({ size, data: await renderPng(src, size, null) });
  }
  fs.writeFileSync(`${OUT}/odina.ico`, buildIco(icoFrames));

  // Press kit: 2048 with sand texture behind the navy arch
  const hero = await sharp(Buffer.from(svgMarkup({ stroke: NAVY, accent: GOLD })))
    .resize(1500, 1500, { fit: 'contain' }).png().toBuffer();
  await sharp(await sandTexture(2048))
    .composite([{ input: hero, gravity: 'center' }])
    .png({ compressionLevel: 9 }).toFile(`${OUT}/press/odina-2048-sand.png`);

  console.log('logo build complete →', OUT);
})();
