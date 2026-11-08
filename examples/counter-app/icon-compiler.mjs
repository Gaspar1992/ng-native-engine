import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { resolve } from 'node:path';
import { deflateSync } from 'node:zlib';

/**
 * Multi-Resolution Icon Compiler (SRP):
 * Decodes 32-bit DIB frames from `favicon.ico` and emits multi-DPI PNGs (`16x16`..`256x256`)
 * for the Freedesktop `hicolor` theme and desktop taskbar integration.
 */
function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc ^= buf[i];
    for (let j = 0; j < 8; j++) {
      crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function makePngChunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, 'ascii');
  const crcBuf = Buffer.alloc(4);
  crcBuf.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
  return Buffer.concat([len, typeBuf, data, crcBuf]);
}

function encodeRgbaToPng(width, height, rgba) {
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;

  const stride = width * 4;
  const rawScanlines = Buffer.alloc((stride + 1) * height);
  for (let y = 0; y < height; y++) {
    const rowStart = y * (stride + 1);
    rawScanlines[rowStart] = 0;
    rgba.copy(rawScanlines, rowStart + 1, y * stride, (y + 1) * stride);
  }

  const idat = deflateSync(rawScanlines);
  return Buffer.concat([
    signature,
    makePngChunk('IHDR', ihdr),
    makePngChunk('IDAT', idat),
    makePngChunk('IEND', Buffer.alloc(0)),
  ]);
}

function extractIcoFrames(icoBuf) {
  const count = icoBuf.readUInt16LE(4);
  const frames = [];
  for (let i = 0; i < count; i++) {
    const entryOff = 6 + i * 16;
    const width = icoBuf[entryOff] || 256;
    const height = icoBuf[entryOff + 1] || 256;
    const bpp = icoBuf.readUInt16LE(entryOff + 6);
    const imgOff = icoBuf.readUInt32LE(entryOff + 12);

    if (bpp !== 32) continue;
    const headerSize = icoBuf.readUInt32LE(imgOff);
    const pixelStart = imgOff + headerSize;
    const rgba = Buffer.alloc(width * height * 4);

    for (let y = 0; y < height; y++) {
      const srcRow = height - 1 - y;
      for (let x = 0; x < width; x++) {
        const srcIdx = pixelStart + (srcRow * width + x) * 4;
        const dstIdx = (y * width + x) * 4;
        rgba[dstIdx] = icoBuf[srcIdx + 2];
        rgba[dstIdx + 1] = icoBuf[srcIdx + 1];
        rgba[dstIdx + 2] = icoBuf[srcIdx];
        rgba[dstIdx + 3] = icoBuf[srcIdx + 3];
      }
    }
    frames.push({ width, height, rgba });
  }
  return frames;
}

function scaleNearest(frame, targetSize) {
  if (frame.width === targetSize && frame.height === targetSize) {
    return frame.rgba;
  }
  const out = Buffer.alloc(targetSize * targetSize * 4);
  for (let y = 0; y < targetSize; y++) {
    const srcY = Math.min(frame.height - 1, Math.floor((y * frame.height) / targetSize));
    for (let x = 0; x < targetSize; x++) {
      const srcX = Math.min(frame.width - 1, Math.floor((x * frame.width) / targetSize));
      const srcIdx = (srcY * frame.width + srcX) * 4;
      const dstIdx = (y * targetSize + x) * 4;
      frame.rgba.copy(out, dstIdx, srcIdx, srcIdx + 4);
    }
  }
  return out;
}

export function buildMultiResolutionIcons(icoAbsPath, appId, outDir, windowTitle) {
  const icoBuf = readFileSync(icoAbsPath);
  const frames = extractIcoFrames(icoBuf);
  if (frames.length === 0) return;

  frames.sort((a, b) => b.width - a.width);
  const targetSizes = [16, 24, 32, 48, 64, 128, 256];
  const localShare = resolve(homedir(), '.local/share');

  for (const size of targetSizes) {
    const exact = frames.find((f) => f.width === size) || frames[0];
    const scaledRgba = scaleNearest(exact, size);
    const pngBytes = encodeRgbaToPng(size, size, scaledRgba);

    const distIconDir = resolve(outDir, 'icons/hicolor', `${size}x${size}`, 'apps');
    mkdirSync(distIconDir, { recursive: true });
    writeFileSync(resolve(distIconDir, `${appId}.png`), pngBytes);

    try {
      const userIconDir = resolve(localShare, 'icons/hicolor', `${size}x${size}`, 'apps');
      mkdirSync(userIconDir, { recursive: true });
      writeFileSync(resolve(userIconDir, `${appId}.png`), pngBytes);
    } catch {
      // Ignore if running in a restricted sandbox without ~/.local/share write access
    }
  }

  try {
    const appsDir = resolve(localShare, 'applications');
    mkdirSync(appsDir, { recursive: true });
    const desktopEntry = [
      '[Desktop Entry]',
      'Type=Application',
      `Name=${windowTitle || appId}`,
      `Icon=${appId}`,
      `StartupWMClass=${appId}`,
      'NoDisplay=true',
      'Terminal=false',
      '',
    ].join('\n');
    writeFileSync(resolve(appsDir, `${appId}.desktop`), desktopEntry, 'utf8');
  } catch {
    // Ignore if ~/.local/share/applications is not writable
  }
}
