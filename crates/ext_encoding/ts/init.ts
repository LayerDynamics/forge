// ext_encoding - TextEncoder/TextDecoder polyfill for Forge runtime
// These APIs are part of the Web standard but not included in a minimal deno_core JsRuntime.
// This extension provides them as globals.

declare const globalThis: {
  TextEncoder: typeof TextEncoderImpl | undefined;
  TextDecoder: typeof TextDecoderImpl | undefined;
  btoa: typeof btoaImpl | undefined;
  atob: typeof atobImpl | undefined;
};

/**
 * TextEncoder - Encodes strings to UTF-8 Uint8Array
 * @see https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder
 */
class TextEncoderImpl {
  readonly encoding = "utf-8";

  encode(input: string = ""): Uint8Array {
    const str = String(input);
    const utf8: number[] = [];

    for (let i = 0; i < str.length; i++) {
      let codePoint = str.charCodeAt(i);

      // Handle surrogate pairs for characters outside BMP (emoji, etc.)
      if (codePoint >= 0xd800 && codePoint <= 0xdbff) {
        const high = codePoint;
        const low = str.charCodeAt(++i);
        if (low >= 0xdc00 && low <= 0xdfff) {
          codePoint = 0x10000 + ((high - 0xd800) << 10) + (low - 0xdc00);
        } else {
          // Invalid surrogate pair - encode replacement character
          utf8.push(0xef, 0xbf, 0xbd);
          i--;
          continue;
        }
      }

      if (codePoint < 0x80) {
        // 1-byte sequence (ASCII)
        utf8.push(codePoint);
      } else if (codePoint < 0x800) {
        // 2-byte sequence
        utf8.push(0xc0 | (codePoint >> 6), 0x80 | (codePoint & 0x3f));
      } else if (codePoint < 0x10000) {
        // 3-byte sequence
        utf8.push(
          0xe0 | (codePoint >> 12),
          0x80 | ((codePoint >> 6) & 0x3f),
          0x80 | (codePoint & 0x3f)
        );
      } else {
        // 4-byte sequence (for code points > U+FFFF)
        utf8.push(
          0xf0 | (codePoint >> 18),
          0x80 | ((codePoint >> 12) & 0x3f),
          0x80 | ((codePoint >> 6) & 0x3f),
          0x80 | (codePoint & 0x3f)
        );
      }
    }

    return new Uint8Array(utf8);
  }

  encodeInto(
    source: string,
    destination: Uint8Array
  ): { read: number; written: number } {
    const encoded = this.encode(source);
    const written = Math.min(encoded.length, destination.length);
    destination.set(encoded.subarray(0, written));

    // Count how many characters were fully encoded
    let read = 0;
    let byteCount = 0;
    for (let i = 0; i < source.length && byteCount < written; i++) {
      const codePoint = source.charCodeAt(i);
      let charBytes: number;

      if (codePoint >= 0xd800 && codePoint <= 0xdbff) {
        // Surrogate pair
        charBytes = 4;
        i++; // Skip low surrogate
      } else if (codePoint < 0x80) {
        charBytes = 1;
      } else if (codePoint < 0x800) {
        charBytes = 2;
      } else {
        charBytes = 3;
      }

      if (byteCount + charBytes <= written) {
        byteCount += charBytes;
        read++;
        if (codePoint >= 0xd800 && codePoint <= 0xdbff) {
          read++; // Count surrogate pair as 2 code units
        }
      } else {
        break;
      }
    }

    return { read, written };
  }
}

/**
 * TextDecoder - Decodes UTF-8 Uint8Array to strings
 * @see https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder
 */
class TextDecoderImpl {
  readonly encoding: string;
  readonly fatal: boolean;
  readonly ignoreBOM: boolean;

  constructor(
    label: string = "utf-8",
    options: { fatal?: boolean; ignoreBOM?: boolean } = {}
  ) {
    // Only UTF-8 is supported
    const normalized = label.toLowerCase().trim();
    if (normalized !== "utf-8" && normalized !== "utf8") {
      throw new RangeError(`The encoding label provided ('${label}') is invalid.`);
    }
    this.encoding = "utf-8";
    this.fatal = options.fatal ?? false;
    this.ignoreBOM = options.ignoreBOM ?? false;
  }

  decode(
    input?: ArrayBuffer | ArrayBufferView,
    _options?: { stream?: boolean }
  ): string {
    if (input === undefined) {
      return "";
    }

    let bytes: Uint8Array;
    if (input instanceof Uint8Array) {
      bytes = input;
    } else if (ArrayBuffer.isView(input)) {
      bytes = new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
    } else if (input instanceof ArrayBuffer) {
      bytes = new Uint8Array(input);
    } else {
      throw new TypeError("Input must be an ArrayBuffer or ArrayBufferView");
    }

    let result = "";
    let i = 0;

    // Skip BOM if present and not ignored
    if (!this.ignoreBOM && bytes.length >= 3) {
      if (bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
        i = 3;
      }
    }

    while (i < bytes.length) {
      const byte1 = bytes[i];

      if (byte1 < 0x80) {
        // 1-byte sequence (ASCII)
        result += String.fromCharCode(byte1);
        i++;
      } else if ((byte1 & 0xe0) === 0xc0) {
        // 2-byte sequence
        if (i + 1 >= bytes.length) {
          if (this.fatal) throw new TypeError("Invalid UTF-8 sequence");
          result += "\ufffd";
          i++;
          continue;
        }
        const byte2 = bytes[i + 1];
        if ((byte2 & 0xc0) !== 0x80) {
          if (this.fatal) throw new TypeError("Invalid UTF-8 sequence");
          result += "\ufffd";
          i++;
          continue;
        }
        const codePoint = ((byte1 & 0x1f) << 6) | (byte2 & 0x3f);
        result += String.fromCharCode(codePoint);
        i += 2;
      } else if ((byte1 & 0xf0) === 0xe0) {
        // 3-byte sequence
        if (i + 2 >= bytes.length) {
          if (this.fatal) throw new TypeError("Invalid UTF-8 sequence");
          result += "\ufffd";
          i++;
          continue;
        }
        const byte2 = bytes[i + 1];
        const byte3 = bytes[i + 2];
        if ((byte2 & 0xc0) !== 0x80 || (byte3 & 0xc0) !== 0x80) {
          if (this.fatal) throw new TypeError("Invalid UTF-8 sequence");
          result += "\ufffd";
          i++;
          continue;
        }
        const codePoint =
          ((byte1 & 0x0f) << 12) | ((byte2 & 0x3f) << 6) | (byte3 & 0x3f);
        result += String.fromCharCode(codePoint);
        i += 3;
      } else if ((byte1 & 0xf8) === 0xf0) {
        // 4-byte sequence (code points > U+FFFF)
        if (i + 3 >= bytes.length) {
          if (this.fatal) throw new TypeError("Invalid UTF-8 sequence");
          result += "\ufffd";
          i++;
          continue;
        }
        const byte2 = bytes[i + 1];
        const byte3 = bytes[i + 2];
        const byte4 = bytes[i + 3];
        if (
          (byte2 & 0xc0) !== 0x80 ||
          (byte3 & 0xc0) !== 0x80 ||
          (byte4 & 0xc0) !== 0x80
        ) {
          if (this.fatal) throw new TypeError("Invalid UTF-8 sequence");
          result += "\ufffd";
          i++;
          continue;
        }
        const codePoint =
          ((byte1 & 0x07) << 18) |
          ((byte2 & 0x3f) << 12) |
          ((byte3 & 0x3f) << 6) |
          (byte4 & 0x3f);

        // Convert to surrogate pair
        const adjusted = codePoint - 0x10000;
        const highSurrogate = 0xd800 + (adjusted >> 10);
        const lowSurrogate = 0xdc00 + (adjusted & 0x3ff);
        result += String.fromCharCode(highSurrogate, lowSurrogate);
        i += 4;
      } else {
        // Invalid byte
        if (this.fatal) throw new TypeError("Invalid UTF-8 sequence");
        result += "\ufffd";
        i++;
      }
    }

    return result;
  }
}

// Install as globals (only if not already defined)
if (typeof globalThis.TextEncoder === "undefined") {
  (globalThis as unknown as Record<string, unknown>).TextEncoder = TextEncoderImpl;
}

if (typeof globalThis.TextDecoder === "undefined") {
  (globalThis as unknown as Record<string, unknown>).TextDecoder = TextDecoderImpl;
}

/**
 * btoa - Encodes a string to Base64
 * @see https://developer.mozilla.org/en-US/docs/Web/API/btoa
 */
function btoaImpl(data: string): string {
  const str = String(data);
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  let result = "";

  // Check for characters outside Latin1 range
  for (let i = 0; i < str.length; i++) {
    if (str.charCodeAt(i) > 255) {
      throw new DOMException(
        "The string to be encoded contains characters outside of the Latin1 range.",
        "InvalidCharacterError"
      );
    }
  }

  let i = 0;
  while (i < str.length) {
    const a = str.charCodeAt(i++);
    const b = i < str.length ? str.charCodeAt(i++) : 0;
    const c = i < str.length ? str.charCodeAt(i++) : 0;

    const triplet = (a << 16) | (b << 8) | c;

    result += chars[(triplet >> 18) & 0x3f];
    result += chars[(triplet >> 12) & 0x3f];
    result += i > str.length + 1 ? "=" : chars[(triplet >> 6) & 0x3f];
    result += i > str.length ? "=" : chars[triplet & 0x3f];
  }

  return result;
}

/**
 * atob - Decodes a Base64 string
 * @see https://developer.mozilla.org/en-US/docs/Web/API/atob
 */
function atobImpl(data: string): string {
  const str = String(data).replace(/[\t\n\f\r ]/g, "");
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

  if (str.length % 4 === 1) {
    throw new DOMException(
      "The string to be decoded is not correctly encoded.",
      "InvalidCharacterError"
    );
  }

  let result = "";
  let i = 0;

  while (i < str.length) {
    const a = chars.indexOf(str[i++]);
    const b = chars.indexOf(str[i++]);
    const c = str[i] === "=" ? 0 : chars.indexOf(str[i++]);
    const d = str[i] === "=" ? 0 : chars.indexOf(str[i++]);

    if (a === -1 || b === -1 || (str[i - 2] !== "=" && c === -1) || (str[i - 1] !== "=" && d === -1)) {
      throw new DOMException(
        "The string to be decoded is not correctly encoded.",
        "InvalidCharacterError"
      );
    }

    const triplet = (a << 18) | (b << 12) | (c << 6) | d;

    result += String.fromCharCode((triplet >> 16) & 0xff);
    if (str[i - 2] !== "=") {
      result += String.fromCharCode((triplet >> 8) & 0xff);
    }
    if (str[i - 1] !== "=") {
      result += String.fromCharCode(triplet & 0xff);
    }
  }

  return result;
}

// Install btoa/atob as globals (only if not already defined)
if (typeof globalThis.btoa === "undefined") {
  (globalThis as unknown as Record<string, unknown>).btoa = btoaImpl;
}

if (typeof globalThis.atob === "undefined") {
  (globalThis as unknown as Record<string, unknown>).atob = atobImpl;
}

// Also export for module usage
export { TextEncoderImpl as TextEncoder, TextDecoderImpl as TextDecoder, btoaImpl as btoa, atobImpl as atob };
