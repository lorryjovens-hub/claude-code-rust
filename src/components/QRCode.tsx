import React, { useState, useEffect, useRef } from 'react';

interface QRCodeProps {
  text: string;
  size?: number;
}

const QRCode: React.FC<QRCodeProps> = ({ text, size = 200 }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const moduleCount = 25;
    const moduleSize = Math.floor(size / moduleCount);
    const actualSize = moduleSize * moduleCount;
    canvas.width = actualSize;
    canvas.height = actualSize;

    const data = generateQRData(text, moduleCount);
    if (!data) {
      ctx.fillStyle = '#f0f0f0';
      ctx.fillRect(0, 0, actualSize, actualSize);
      ctx.fillStyle = '#333';
      ctx.font = '12px sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText('QR code unavailable', actualSize / 2, actualSize / 2 - 8);
      ctx.fillText('use link below', actualSize / 2, actualSize / 2 + 12);
      return;
    }

    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, actualSize, actualSize);

    for (let row = 0; row < moduleCount; row++) {
      for (let col = 0; col < moduleCount; col++) {
        if (data[row * moduleCount + col]) {
          ctx.fillStyle = '#000000';
          ctx.fillRect(col * moduleSize, row * moduleSize, moduleSize, moduleSize);
        }
      }
    }
  }, [text, size]);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      const textArea = document.createElement('textarea');
      textArea.value = text;
      document.body.appendChild(textArea);
      textArea.select();
      document.execCommand('copy');
      document.body.removeChild(textArea);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="flex flex-col items-center gap-3">
      <canvas
        ref={canvasRef}
        className="border border-claude-border rounded-lg"
        style={{ width: size, height: size }}
      />
      <div className="flex items-center gap-2 w-full max-w-full">
        <input
          type="text"
          value={text}
          readOnly
          className="flex-1 px-3 py-2 bg-claude-input border border-claude-border rounded-lg text-claude-text text-[13px] truncate focus:outline-none"
        />
        <button
          onClick={handleCopy}
          className={`px-4 py-2 rounded-lg text-[13px] font-medium transition-colors ${
            copied
              ? 'bg-green-500/20 text-green-400 border border-green-500/30'
              : 'bg-blue-500/20 text-blue-400 border border-blue-500/30 hover:bg-blue-500/30'
          }`}
        >
          {copied ? '已复制' : '复制'}
        </button>
      </div>
    </div>
  );
};

function generateQRData(text: string, moduleCount: number): boolean[] | null {
  try {
    const segments = encodeQRData(text);
    const version = getVersion(segments.length, moduleCount);
    if (version === undefined) return null;
    const totalCodewords = versionToTotalCodewords(version);
    const dataStr = bitsToString(segments);

    const requiredBits = totalCodewords * 8;

    let dataBits = dataStr;
    const terminatorBits = Math.min(4, requiredBits - dataBits.length);
    if (terminatorBits > 0) {
      dataBits += '0'.repeat(terminatorBits);
    }

    while (dataBits.length % 8 !== 0) {
      dataBits += '0';
    }

    const padBytes = [0xEC, 0x11];
    let padIdx = 0;
    while (dataBits.length < 271 * 8) {
      dataBits += padBytes[padIdx % 2].toString(2).padStart(8, '0');
      padIdx++;
    }

    dataBits = dataBits.substring(0, 271 * 8);

    const dataCodewords: number[] = [];
    for (let i = 0; i < dataBits.length; i += 8) {
      dataCodewords.push(parseInt(dataBits.substring(i, i + 8), 2));
    }

    const blocks = interleaveData(dataCodewords, version);
    const ecCodewords = generateErrorCorrection(blocks, version);
    const allCodewords = interleaveFinal(blocks, ecCodewords, version);

    let finalBits = '';
    for (const cw of allCodewords) {
      finalBits += cw.toString(2).padStart(8, '0');
    }

    const modules: boolean[] = new Array(moduleCount * moduleCount).fill(false);

    for (let i = 0; i < moduleCount; i++) {
      for (let j = 0; j < moduleCount; j++) {
        modules[i * moduleCount + j] = false;
      }
    }

    const finderPatterns = [
      [0, 0],
      [0, moduleCount - 7],
      [moduleCount - 7, 0],
    ];

    for (const [r, c] of finderPatterns) {
      for (let i = 0; i < 7; i++) {
        for (let j = 0; j < 7; j++) {
          const v =
            (i === 0 || i === 6 || j === 0 || j === 6) ||
            (i >= 2 && i <= 4 && j >= 2 && j <= 4);
          modules[(r + i) * moduleCount + (c + j)] = v;
        }
      }
    }

    const alignPositions = getAlignmentPatternCenters(version);
    for (const ar of alignPositions) {
      for (const ac of alignPositions) {
        if (isFinderZone(ar, ac, moduleCount, finderPatterns)) continue;
        for (let i = -2; i <= 2; i++) {
          for (let j = -2; j <= 2; j++) {
            const v = i === -2 || i === 2 || j === -2 || j === 2 || (i === 0 && j === 0);
            const rr = ar + i;
            const cc = ac + j;
            if (rr >= 0 && rr < moduleCount && cc >= 0 && cc < moduleCount) {
              modules[rr * moduleCount + cc] = v;
            }
          }
        }
      }
    }

    for (let i = 0; i < moduleCount; i++) {
      modules[i * moduleCount + 6] = i % 2 === 0;
    }
    for (let i = 0; i < moduleCount; i++) {
      modules[6 * moduleCount + i] = i % 2 === 0;
    }

    if (version >= 2) {
      const formatInfo = getFormatInfo();
      for (let i = 0; i < 6; i++) {
        modules[i * moduleCount + 8] = formatInfo[i];
      }
      for (let i = 0; i < 6; i++) {
        modules[8 * moduleCount + (moduleCount - 1 - i)] = formatInfo[i];
      }
      modules[7 * moduleCount + 8] = formatInfo[6];
      modules[8 * moduleCount + (moduleCount - 7)] = formatInfo[7];
      modules[8 * moduleCount + 8] = formatInfo[8];

      if (version >= 7) {
        const verInfo = getVersionInfo(version);
        for (let i = 0; i < 6; i++) {
          for (let j = 0; j < 3; j++) {
            modules[(moduleCount - 11 + j) * moduleCount + i] = verInfo[i * 3 + j];
            modules[i * moduleCount + (moduleCount - 11 + j)] = verInfo[i * 3 + j];
          }
        }
      }
    }

    let bitIdx = 0;
    let goingUp = true;
    let col = moduleCount - 1;

    while (col > 0) {
      if (col === 6) col--;
      for (let row = moduleCount - 1; row >= 0; row--) {
        const r = goingUp ? row : moduleCount - 1 - row;
        for (let cOffset = 0; cOffset < 2; cOffset++) {
          const cc = col - cOffset;
          if (isReserved(r, cc, moduleCount, finderPatterns)) continue;
          if (bitIdx < finalBits.length) {
            modules[r * moduleCount + cc] = finalBits[bitIdx] === '1';
            bitIdx++;
          }
        }
      }
      goingUp = !goingUp;
      col -= 2;
    }

    const bestMask = selectMask(modules, moduleCount, finderPatterns, alignPositions, version);
    applyMask(modules, moduleCount, bestMask);

    const formatBits = getFormatBits(bestMask);
    for (let i = 0; i < 6; i++) {
      modules[i * moduleCount + 8] = formatBits[i];
    }
    for (let i = 0; i < 6; i++) {
      modules[8 * moduleCount + (moduleCount - 1 - i)] = formatBits[i];
    }
    modules[7 * moduleCount + 8] = formatBits[6];
    modules[8 * moduleCount + (moduleCount - 7)] = formatBits[7];
    modules[8 * moduleCount + 8] = formatBits[8];

    return modules;
  } catch {
    return null;
  }
}

function encodeQRData(text: string): number[] {
  const bits: number[] = [];
  bits.push(0, 1, 0, 0);

  const charCount = text.length;
  const countBits = charCount.toString(2).padStart(8, '0');
  for (const ch of countBits) {
    bits.push(ch === '1' ? 1 : 0);
  }

  for (const ch of text) {
    const code = ch.charCodeAt(0);
    if (code < 128) {
      const byte = code.toString(2).padStart(8, '0');
      for (const b of byte) {
        bits.push(b === '1' ? 1 : 0);
      }
    } else {
      const utf8 = encodeUTF8(ch);
      for (const byte of utf8) {
        const b = byte.toString(2).padStart(8, '0');
        for (const c of b) {
          bits.push(c === '1' ? 1 : 0);
        }
      }
    }
  }

  return bits;
}

function encodeUTF8(ch: string): number[] {
  const code = ch.charCodeAt(0);
  if (code < 0x80) {
    return [code];
  } else if (code < 0x800) {
    return [0xc0 | (code >> 6), 0x80 | (code & 0x3f)];
  } else {
    return [0xe0 | (code >> 12), 0x80 | ((code >> 6) & 0x3f), 0x80 | (code & 0x3f)];
  }
}

function bitsToString(bits: number[]): string {
  return bits.map((b) => (b ? '1' : '0')).join('');
}

function getVersion(dataLen: number, moduleCount: number): number | undefined {
  const versions = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
  ];
  const capacities = [19, 34, 55, 80, 108, 136, 156, 194, 232, 274, 324, 370, 428, 461, 523, 589, 647, 721, 795, 861];
  const byteCount = Math.ceil(dataLen / 8);

  for (let i = 0; i < versions.length; i++) {
    const expectedModuleCount = 17 + 4 * versions[i];
    if (expectedModuleCount === moduleCount && byteCount <= capacities[i]) {
      return versions[i];
    }
  }
  return undefined;
}

function versionToTotalCodewords(version: number): number {
  const totalCodewordsTable: Record<number, number> = {
    1: 26, 2: 44, 3: 70, 4: 100, 5: 134, 6: 172, 7: 196, 8: 242, 9: 292, 10: 346,
  };
  return totalCodewordsTable[version] || 26;
}

function getAlignmentPatternCenters(version: number): number[] {
  if (version === 1) return [];
  const numAlign = Math.floor(version / 7) + 2;
  const step = Math.floor(version / 7) === 0 ? 28 : (version === 0 ? 0 : (26 + 4 * version) / (numAlign - 1));
  const centers: number[] = [6];
  const last = 6 + (version === 2 ? 16 : version * 4 + 10);
  if (numAlign === 2) {
    if (version === 2) centers.push(22);
    else centers.push(last);
  } else {
    const stepVal = Math.floor((last - 6) / (numAlign - 1));
    for (let i = 1; i <= numAlign - 1; i++) {
      centers.push(6 + i * stepVal);
    }
  }
  return [...new Set(centers)];
}

function isFinderZone(r: number, c: number, moduleCount: number, finderPatterns: number[][]): boolean {
  for (const [fr, fc] of finderPatterns) {
    if (r >= fr - 2 && r <= fr + 8 && c >= fc - 2 && c <= fc + 8) return true;
  }
  return false;
}

function isReserved(r: number, c: number, moduleCount: number, finderPatterns: number[][]): boolean {
  if (isFinderZone(r, c, moduleCount, finderPatterns)) return true;
  if (r === 6 || c === 6) return true;
  if (r >= 0 && r <= 8 && c >= 0 && c <= 8) return true;
  if (r >= 0 && r <= 8 && c >= moduleCount - 8 && c <= moduleCount - 1) return true;
  if (r >= moduleCount - 8 && r <= moduleCount - 1 && c >= 0 && c <= 8) return true;
  return false;
}

function getFormatInfo(): boolean[] {
  return [true, false, true, false, false, false, false, false, true];
}

function getVersionInfo(version: number): boolean[] {
  const table: Record<number, string> = {
    7: '000111110010010100',
    8: '001000010110111100',
    9: '001001101010011001',
    10: '001010010011010011',
    11: '001011101111110110',
    12: '001100011101100010',
  };
  const bits = (table[version] || '000111110010010100').split('');
  return bits.map((b) => b === '1');
}

function interleaveData(dataCodewords: number[], version: number): number[][] {
  const blocks: number[][] = [];
  let idx = 0;
  const blockCount = version <= 9 ? 1 : 2;
  const codewordsPerBlock = Math.floor(dataCodewords.length / blockCount);

  for (let b = 0; b < blockCount; b++) {
    const block: number[] = [];
    for (let i = 0; i < codewordsPerBlock; i++) {
      block.push(dataCodewords[idx] || 0);
      idx++;
    }
    blocks.push(block);
  }

  while (idx < dataCodewords.length) {
    blocks[0].push(dataCodewords[idx]);
    idx++;
  }

  return blocks;
}

function generateErrorCorrection(blocks: number[][], version: number): number[][] {
  const ecCodewordsPerBlock = version <= 9 ? 7 : 10;
  const ecBlocks: number[][] = [];

  for (const block of blocks) {
    const ec: number[] = new Array(ecCodewordsPerBlock).fill(0);
    const generator = getGeneratorPolynomial(ecCodewordsPerBlock);
    const msgPoly = [...block, ...new Array(ecCodewordsPerBlock).fill(0)];

    for (let i = 0; i < block.length; i++) {
      const factor = msgPoly[i];
      if (factor === 0) continue;
      const logFactor = LOG_TABLE[factor];
      for (let j = 0; j < generator.length; j++) {
        const genVal = generator[j];
        msgPoly[i + j] ^= EXP_TABLE[(logFactor + genVal) % 255];
      }
    }

    for (let i = 0; i < ecCodewordsPerBlock; i++) {
      ec[i] = msgPoly[block.length + i];
    }
    ecBlocks.push(ec);
  }

  return ecBlocks;
}

const EXP_TABLE: number[] = [];
const LOG_TABLE: number[] = new Array(256).fill(0);

(function initGF256() {
  let x = 1;
  for (let i = 0; i < 255; i++) {
    EXP_TABLE[i] = x;
    LOG_TABLE[x] = i;
    x <<= 1;
    if (x >= 256) x ^= 0x11d;
  }
  EXP_TABLE[255] = EXP_TABLE[0];
})();

function getGeneratorPolynomial(degree: number): number[] {
  let poly = [0];
  for (let i = 1; i < degree; i++) {
    const newPoly = [poly[0]];
    for (let j = 0; j < poly.length - 1; j++) {
      const sum = (poly[j] + i) % 255;
      newPoly.push(LOG_TABLE[EXP_TABLE[poly[j + 1]] ^ EXP_TABLE[sum]]);
    }
    newPoly.push((poly[poly.length - 1] + i) % 255);
    poly = newPoly;
  }
  return poly;
}

function interleaveFinal(blocks: number[][], ecBlocks: number[][], version: number): number[] {
  const result: number[] = [];
  const maxLen = Math.max(...blocks.map((b) => b.length));

  for (let i = 0; i < maxLen; i++) {
    for (const block of blocks) {
      if (i < block.length) result.push(block[i]);
    }
  }

  const maxEcLen = Math.max(...ecBlocks.map((b) => b.length));
  for (let i = 0; i < maxEcLen; i++) {
    for (const ec of ecBlocks) {
      if (i < ec.length) result.push(ec[i]);
    }
  }

  return result;
}

const MASK_PATTERNS = [
  (i: number, j: number) => (i + j) % 2 === 0,
  (i: number, _j: number) => i % 2 === 0,
  (_i: number, j: number) => j % 3 === 0,
  (i: number, j: number) => (i + j) % 3 === 0,
  (i: number, j: number) => (Math.floor(i / 2) + Math.floor(j / 3)) % 2 === 0,
  (i: number, j: number) => (i * j) % 2 + (i * j) % 3 === 0,
  (i: number, j: number) => ((i * j) % 2 + (i * j) % 3) % 2 === 0,
  (i: number, j: number) => ((i + j) % 2 + (i * j) % 3) % 2 === 0,
];

function selectMask(modules: boolean[], moduleCount: number, finderPatterns: number[][], alignPositions: number[], version: number): number {
  let bestScore = Infinity;
  let bestMask = 0;

  for (let maskIdx = 0; maskIdx < 8; maskIdx++) {
    const testModules = [...modules];
    for (let r = 0; r < moduleCount; r++) {
      for (let c = 0; c < moduleCount; c++) {
        if (isReserved(r, c, moduleCount, finderPatterns)) continue;
        if (MASK_PATTERNS[maskIdx](r, c)) {
          testModules[r * moduleCount + c] = !testModules[r * moduleCount + c];
        }
      }
    }
    const score = evaluateMask(testModules, moduleCount);
    if (score < bestScore) {
      bestScore = score;
      bestMask = maskIdx;
    }
  }

  return bestMask;
}

function evaluateMask(modules: boolean[], size: number): number {
  let penalty = 0;

  for (let r = 0; r < size; r++) {
    let run = 0;
    let last = false;
    for (let c = 0; c < size; c++) {
      const v = modules[r * size + c];
      if (v === last) {
        run++;
      } else {
        if (run >= 5) penalty += 3 + (run - 5);
        run = 1;
        last = v;
      }
    }
    if (run >= 5) penalty += 3 + (run - 5);
  }

  for (let c = 0; c < size; c++) {
    let run = 0;
    let last = false;
    for (let r = 0; r < size; r++) {
      const v = modules[r * size + c];
      if (v === last) {
        run++;
      } else {
        if (run >= 5) penalty += 3 + (run - 5);
        run = 1;
        last = v;
      }
    }
    if (run >= 5) penalty += 3 + (run - 5);
  }

  for (let r = 0; r < size - 1; r++) {
    for (let c = 0; c < size - 1; c++) {
      const v = modules[r * size + c];
      if (
        v === modules[r * size + c + 1] &&
        v === modules[(r + 1) * size + c] &&
        v === modules[(r + 1) * size + c + 1]
      ) {
        penalty += 3;
      }
    }
  }

  const totalModules = size * size;
  let darkCount = 0;
  for (const m of modules) {
    if (m) darkCount++;
  }
  const percent = (darkCount / totalModules) * 100;
  const prev = Math.floor(percent / 5) * 5;
  const next = prev + 5;
  const dist = Math.min(Math.abs(prev - 50), Math.abs(next - 50));
  penalty += (dist / 5) * 10;

  return penalty;
}

function applyMask(modules: boolean[], moduleCount: number, maskIdx: number): void {
  const finderPatterns: number[][] = [[0, 0], [0, moduleCount - 7], [moduleCount - 7, 0]];
  for (let r = 0; r < moduleCount; r++) {
    for (let c = 0; c < moduleCount; c++) {
      if (isReserved(r, c, moduleCount, finderPatterns)) continue;
      if (MASK_PATTERNS[maskIdx](r, c)) {
        modules[r * moduleCount + c] = !modules[r * moduleCount + c];
      }
    }
  }
}

function getFormatBits(maskPattern: number): boolean[] {
  const formatBits = [true, false, true, false, false, false, false, false, true, false, false, false, false, false, false];
  const maskBits = maskPattern.toString(2).padStart(3, '0');
  formatBits[12] = maskBits[0] === '1';
  formatBits[13] = maskBits[1] === '1';
  formatBits[14] = maskBits[2] === '1';

  let formatVal = 0;
  for (let i = 0; i < 15; i++) {
    if (formatBits[i]) formatVal |= 1 << (14 - i);
  }

  let gfVal = formatVal;
  for (let i = 0; i <= 4; i++) {
    if (gfVal & (1 << (14 - i))) {
      gfVal ^= 0x537;
    }
    gfVal = ((gfVal & 0x7fff) << 1);
  }

  formatVal = (formatVal << 10) | gfVal;

  const result: boolean[] = [];
  for (let i = 0; i < 15; i++) {
    result.push(((formatVal >> (24 - i)) & 1) !== 0);
  }
  return result;
}

export default QRCode;