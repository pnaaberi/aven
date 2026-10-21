import assert from 'node:assert/strict';
import sharp from 'sharp';

function isPatched(version) {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
  if (!match) return false;
  const [major, minor, patch] = match.slice(1).map(Number);
  return major > 0 || minor > 35 || (minor === 35 && patch >= 5);
}

assert.equal(isPatched('0.35.4'), false, 'detect the vulnerable baseline');
assert.equal(isPatched('0.35.5'), true);
assert.equal(isPatched('0.36.0'), true);
assert.equal(isPatched('0.35.5-beta'), false);
assert.equal(isPatched('invalid'), false);
assert.ok(isPatched(sharp.versions.sharp), 'Sharp must include all three security fixes');

const source = sharp({
  create: { width: 2, height: 2, channels: 4, background: '#ffffff' },
});
const png = await source.png().toBuffer();
for (const format of ['png', 'webp', 'avif']) {
  const encoded = await sharp(png).toFormat(format).toBuffer();
  const resized = await sharp(encoded).resize(1, 1).png().toBuffer();
  const metadata = await sharp(resized).metadata();
  assert.equal(metadata.width, 1);
  assert.equal(metadata.height, 1);
}
const svg = Buffer.from('<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2"><rect width="2" height="2"/></svg>');
const rendered = await sharp(svg).resize(1, 1).png().toBuffer();
assert.equal((await sharp(rendered).metadata()).width, 1);
await assert.rejects(sharp(Buffer.from('not an image')).metadata());
console.log('Patched Sharp; PNG/WebP/AVIF/SVG round trips and invalid-input rejection passed.');
