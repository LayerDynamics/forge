---
title: "runtime:image_tools"
description: "Image manipulation and format conversion for Forge applications"
slug: api/runtime-image_tools
---

Image manipulation and format conversion utilities for Forge applications. Process PNG, SVG, and WebP images with operations for loading, saving, converting between formats, and applying transformations.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_image_tools](/docs/crates/ext-image-tools) for implementation details.

## Features

### PNG Operations
- Load and parse PNG images
- Get image dimensions and color information
- Save with configurable compression
- Optimize by removing metadata

### SVG Operations
- Parse SVG files and extract dimensions
- Get viewBox information
- Render SVG to PNG at any resolution

### WebP Operations
- Encode images to WebP format
- Decode WebP back to PNG
- Query WebP image information
- Lossy and lossless compression

### Format Conversion
- SVG to PNG rasterization
- PNG to ICO (multi-size icons)
- ICO extraction to individual PNGs
- Complete favicon set generation
- PNG to WebP conversion

### Image Transforms
- Resize to exact dimensions
- Scale by factor
- Crop regions
- Rotate (90, 180, 270 degrees)
- Flip horizontal/vertical

## Import

```typescript
import {
  // PNG Operations
  pngInfo,
  pngLoad,
  pngSave,
  pngOptimize,
  // SVG Operations
  svgInfo,
  svgLoad,
  // WebP Operations
  webpEncode,
  webpDecode,
  webpInfo,
  // Conversions
  svgToPng,
  pngToIco,
  icoExtract,
  faviconCreate,
  pngToWebp,
  // Transforms
  resize,
  scale,
  crop,
  rotate,
  flip,
  // Types
  type ImageInfo,
  type SvgInfo,
  type ViewBox,
  type WebPInfo,
  type FaviconSet,
  type PngSaveOptions,
  type FilterType,
  type FlipDirection,
} from "runtime:image_tools";
```

## API Reference

### PNG Operations

#### pngInfo(data)

Get information about a PNG image without fully decoding it.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `Uint8Array` | PNG image bytes |

**Returns:** `ImageInfo`

**Example:**

```typescript
import { pngInfo } from "runtime:image_tools";
import { readFile } from "runtime:fs";

const pngData = await readFile("./image.png");
const info = pngInfo(pngData);

console.log(`Size: ${info.width}x${info.height}`);
console.log(`Format: ${info.format}`);
console.log(`Has alpha: ${info.hasAlpha}`);
```

---

#### pngLoad(data)

Load a PNG image and decode its contents.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `Uint8Array` | PNG image bytes |

**Returns:** `ImageInfo`

**Example:**

```typescript
import { pngLoad } from "runtime:image_tools";

const info = pngLoad(imageData);
console.log(`Loaded ${info.width}x${info.height} ${info.colorType} image`);
```

---

#### pngSave(data, options?)

Save/re-encode a PNG image with optional compression settings.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `Uint8Array` | Source image bytes |
| `options` | `PngSaveOptions` | Optional save settings |

**Returns:** `Uint8Array` - PNG bytes

**Example:**

```typescript
import { pngSave } from "runtime:image_tools";

// Re-encode with maximum compression
const compressed = pngSave(imageData, { compression: 9 });
console.log(`Compressed size: ${compressed.byteLength} bytes`);
```

---

#### pngOptimize(data)

Optimize a PNG by re-encoding (removes metadata, applies compression).

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `Uint8Array` | PNG image bytes |

**Returns:** `Uint8Array` - Optimized PNG bytes

**Example:**

```typescript
import { pngOptimize } from "runtime:image_tools";

const original = await readFile("./screenshot.png");
const optimized = pngOptimize(original);

console.log(`Original: ${original.byteLength} bytes`);
console.log(`Optimized: ${optimized.byteLength} bytes`);
console.log(`Saved: ${original.byteLength - optimized.byteLength} bytes`);
```

---

### SVG Operations

#### svgInfo(svgData)

Get information about an SVG without rendering.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `svgData` | `string` | SVG string content |

**Returns:** `SvgInfo`

**Example:**

```typescript
import { svgInfo } from "runtime:image_tools";

const svg = `<svg width="100" height="100" viewBox="0 0 100 100">...</svg>`;
const info = svgInfo(svg);

console.log(`SVG size: ${info.width}x${info.height}`);
if (info.viewBox) {
  console.log(`ViewBox: ${info.viewBox.x}, ${info.viewBox.y}, ${info.viewBox.width}, ${info.viewBox.height}`);
}
```

---

#### svgLoad(svgData)

Load and parse an SVG document.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `svgData` | `string` | SVG string content |

**Returns:** `SvgInfo`

**Example:**

```typescript
import { svgLoad } from "runtime:image_tools";
import { readText } from "runtime:fs";

const svgContent = await readText("./icon.svg");
const info = svgLoad(svgContent);
console.log(`Loaded SVG: ${info.width}x${info.height}`);
```

---

### WebP Operations

#### webpEncode(data, quality?)

Encode an image as WebP format.

> **Note:** WebP is intended for app asset optimization only, NOT for icons or bundle-specific formats.

**Parameters:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `data` | `Uint8Array` | - | Source PNG bytes |
| `quality` | `number` | `80` | Quality level (0-100, 100 = lossless) |

**Returns:** `Uint8Array` - WebP bytes

**Example:**

```typescript
import { webpEncode } from "runtime:image_tools";

// Lossy compression at 80% quality
const webp = webpEncode(pngData, 80);

// Lossless compression
const lossless = webpEncode(pngData, 100);
```

---

#### webpDecode(data)

Decode WebP to PNG format.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `Uint8Array` | WebP image bytes |

**Returns:** `Uint8Array` - PNG bytes

**Example:**

```typescript
import { webpDecode } from "runtime:image_tools";
import { writeFile } from "runtime:fs";

const png = webpDecode(webpData);
await writeFile("./converted.png", png);
```

---

#### webpInfo(data)

Get information about a WebP image.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `Uint8Array` | WebP image bytes |

**Returns:** `WebPInfo`

**Example:**

```typescript
import { webpInfo } from "runtime:image_tools";

const info = webpInfo(webpData);
console.log(`WebP: ${info.width}x${info.height}`);
console.log(`Has alpha: ${info.hasAlpha}`);
console.log(`Lossless: ${info.isLossless}`);
```

---

### Conversion Operations

#### svgToPng(svgData, width, height)

Convert SVG to PNG at specified dimensions.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `svgData` | `string` | SVG string content |
| `width` | `number` | Target width in pixels |
| `height` | `number` | Target height in pixels |

**Returns:** `Uint8Array` - PNG bytes

**Example:**

```typescript
import { svgToPng } from "runtime:image_tools";
import { readText, writeFile } from "runtime:fs";

const svg = await readText("./logo.svg");

// Render at 512x512 for high-DPI displays
const png = svgToPng(svg, 512, 512);
await writeFile("./logo-512.png", png);

// Render at multiple sizes
const sizes = [64, 128, 256, 512];
for (const size of sizes) {
  const rendered = svgToPng(svg, size, size);
  await writeFile(`./logo-${size}.png`, rendered);
}
```

---

#### pngToIco(pngData)

Convert PNG image(s) to ICO format.

If a single PNG is provided, it will be resized to standard ICO sizes. If multiple PNGs are provided, they should be different sizes.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `pngData` | `Uint8Array[]` | Array of PNG image bytes |

**Returns:** `Uint8Array` - ICO file bytes

**Example:**

```typescript
import { pngToIco, resize } from "runtime:image_tools";

// From a single source image
const source = await readFile("./icon-256.png");
const ico = pngToIco([source]);
await writeFile("./app.ico", ico);

// From multiple pre-sized images
const icon16 = resize(source, 16, 16);
const icon32 = resize(source, 32, 32);
const icon48 = resize(source, 48, 48);
const icon256 = source;

const multiIco = pngToIco([icon16, icon32, icon48, icon256]);
await writeFile("./app-multi.ico", multiIco);
```

---

#### icoExtract(icoData)

Extract all images from an ICO file.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `icoData` | `Uint8Array` | ICO file bytes |

**Returns:** `Uint8Array[]` - Array of PNG bytes (one per size in the ICO)

**Example:**

```typescript
import { icoExtract, pngInfo } from "runtime:image_tools";

const icoData = await readFile("./app.ico");
const images = icoExtract(icoData);

console.log(`Extracted ${images.length} images from ICO`);
for (const png of images) {
  const info = pngInfo(png);
  console.log(`  ${info.width}x${info.height}`);
}
```

---

#### faviconCreate(pngData)

Create a complete favicon set from a source PNG.

Generates:
- 16x16, 32x32, 48x48 PNGs
- 180x180 Apple touch icon
- Multi-size ICO file

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `pngData` | `Uint8Array` | Source PNG (should be at least 180x180, square) |

**Returns:** `FaviconSet`

**Example:**

```typescript
import { faviconCreate } from "runtime:image_tools";
import { readFile, writeFile } from "runtime:fs";

const source = await readFile("./icon-512.png");
const favicons = faviconCreate(source);

// Save all favicon files
await writeFile("./web/favicon-16.png", favicons.favicon16);
await writeFile("./web/favicon-32.png", favicons.favicon32);
await writeFile("./web/favicon-48.png", favicons.favicon48);
await writeFile("./web/apple-touch-icon.png", favicons.apple180);
await writeFile("./web/favicon.ico", favicons.ico);

console.log("Favicon set generated!");
```

---

#### pngToWebp(data, quality?)

Convert PNG to WebP format.

**Parameters:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `data` | `Uint8Array` | - | PNG image bytes |
| `quality` | `number` | `80` | Quality level (0-100, 100 = lossless) |

**Returns:** `Uint8Array` - WebP bytes

**Example:**

```typescript
import { pngToWebp } from "runtime:image_tools";

const png = await readFile("./photo.png");
const webp = pngToWebp(png, 85);
await writeFile("./photo.webp", webp);

console.log(`PNG: ${png.byteLength} bytes`);
console.log(`WebP: ${webp.byteLength} bytes`);
```

---

### Transform Operations

#### resize(data, width, height, filter?)

Resize image to exact dimensions.

**Parameters:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `data` | `Uint8Array` | - | Source image bytes |
| `width` | `number` | - | Target width in pixels |
| `height` | `number` | - | Target height in pixels |
| `filter` | `FilterType` | `"Lanczos3"` | Resize filter algorithm |

**Returns:** `Uint8Array` - Resized PNG bytes

**Example:**

```typescript
import { resize } from "runtime:image_tools";

const original = await readFile("./photo.png");

// High-quality resize with Lanczos3 (default)
const thumbnail = resize(original, 200, 200);

// Fast resize with nearest neighbor
const preview = resize(original, 100, 100, "Nearest");

// Balanced quality with bilinear
const medium = resize(original, 400, 400, "Bilinear");
```

---

#### scale(data, factor)

Scale image by a factor.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `Uint8Array` | Source image bytes |
| `factor` | `number` | Scale factor (0.5 = half size, 2.0 = double) |

**Returns:** `Uint8Array` - Scaled PNG bytes

**Example:**

```typescript
import { scale, pngInfo } from "runtime:image_tools";

const original = await readFile("./image.png");
const info = pngInfo(original);
console.log(`Original: ${info.width}x${info.height}`);

// Create @2x and @3x versions
const double = scale(original, 2.0);
const triple = scale(original, 3.0);

// Create thumbnail at 50% size
const half = scale(original, 0.5);
```

---

#### crop(data, x, y, width, height)

Crop a region from an image.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `Uint8Array` | Source image bytes |
| `x` | `number` | Left edge of crop region |
| `y` | `number` | Top edge of crop region |
| `width` | `number` | Width of crop region |
| `height` | `number` | Height of crop region |

**Returns:** `Uint8Array` - Cropped PNG bytes

**Example:**

```typescript
import { crop, pngInfo } from "runtime:image_tools";

const photo = await readFile("./photo.png");
const info = pngInfo(photo);

// Crop center square
const size = Math.min(info.width, info.height);
const x = (info.width - size) / 2;
const y = (info.height - size) / 2;
const square = crop(photo, x, y, size, size);

// Crop specific region
const region = crop(photo, 100, 50, 400, 300);
```

---

#### rotate(data, degrees)

Rotate image by 90, 180, or 270 degrees.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `Uint8Array` | Source image bytes |
| `degrees` | `90 \| 180 \| 270` | Rotation angle |

**Returns:** `Uint8Array` - Rotated PNG bytes

**Example:**

```typescript
import { rotate } from "runtime:image_tools";

const photo = await readFile("./photo.png");

// Rotate clockwise
const rotated90 = rotate(photo, 90);

// Flip upside down
const rotated180 = rotate(photo, 180);

// Rotate counter-clockwise
const rotated270 = rotate(photo, 270);
```

---

#### flip(data, direction)

Flip image horizontally or vertically.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `Uint8Array` | Source image bytes |
| `direction` | `FlipDirection` | `"Horizontal"` or `"Vertical"` |

**Returns:** `Uint8Array` - Flipped PNG bytes

**Example:**

```typescript
import { flip } from "runtime:image_tools";

const photo = await readFile("./selfie.png");

// Mirror (flip horizontally)
const mirrored = flip(photo, "Horizontal");

// Flip upside down
const flipped = flip(photo, "Vertical");
```

## Type Definitions

### ImageInfo

```typescript
interface ImageInfo {
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** Image format (e.g., "PNG") */
  format: string;
  /** Whether the image has an alpha channel */
  hasAlpha: boolean;
  /** Color type (e.g., "RGBA", "RGB", "Grayscale") */
  colorType: string;
}
```

### SvgInfo

```typescript
interface SvgInfo {
  /** Width from SVG width attribute */
  width: number;
  /** Height from SVG height attribute */
  height: number;
  /** Optional viewBox definition */
  viewBox?: ViewBox;
}
```

### ViewBox

```typescript
interface ViewBox {
  /** Left edge of viewBox */
  x: number;
  /** Top edge of viewBox */
  y: number;
  /** Width of viewBox */
  width: number;
  /** Height of viewBox */
  height: number;
}
```

### WebPInfo

```typescript
interface WebPInfo {
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** Whether the image has an alpha channel */
  hasAlpha: boolean;
  /** Whether encoded as lossless WebP */
  isLossless: boolean;
}
```

### FaviconSet

```typescript
interface FaviconSet {
  /** 16x16 favicon PNG */
  favicon16: Uint8Array;
  /** 32x32 favicon PNG */
  favicon32: Uint8Array;
  /** 48x48 favicon PNG (high-DPI) */
  favicon48: Uint8Array;
  /** 180x180 Apple touch icon PNG */
  apple180: Uint8Array;
  /** Multi-size ICO file */
  ico: Uint8Array;
}
```

### PngSaveOptions

```typescript
interface PngSaveOptions {
  /** Compression level (0-9, default 6) */
  compression?: number;
}
```

### FilterType

```typescript
type FilterType = "Nearest" | "Bilinear" | "Lanczos3";
```

| Filter | Quality | Speed | Use Case |
|--------|---------|-------|----------|
| `Nearest` | Low | Fastest | Pixel art, previews |
| `Bilinear` | Medium | Fast | General purpose |
| `Lanczos3` | High | Slower | High-quality scaling |

### FlipDirection

```typescript
type FlipDirection = "Horizontal" | "Vertical";
```

## Lifecycle Hooks

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:image_tools";

onBefore("resize", () => {
  console.log("Resizing image...");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:image_tools";

onAfter("pngOptimize", () => {
  console.log("PNG optimized");
});
```

**Available operation names:**
- PNG: `"pngInfo"`, `"pngLoad"`, `"pngSave"`, `"pngOptimize"`
- SVG: `"svgInfo"`, `"svgLoad"`
- WebP: `"webpEncode"`, `"webpDecode"`, `"webpInfo"`
- Conversions: `"svgToPng"`, `"pngToIco"`, `"icoExtract"`, `"faviconCreate"`, `"pngToWebp"`
- Transforms: `"resize"`, `"scale"`, `"crop"`, `"rotate"`, `"flip"`

## Complete Examples

### Image Processing Pipeline

```typescript
import {
  pngLoad,
  resize,
  pngOptimize,
  pngToWebp,
} from "runtime:image_tools";
import { readFile, writeFile } from "runtime:fs";

interface ProcessingOptions {
  maxWidth: number;
  maxHeight: number;
  webpQuality: number;
  outputFormats: ("png" | "webp")[];
}

async function processImage(
  inputPath: string,
  outputDir: string,
  options: ProcessingOptions
): Promise<void> {
  // Load source image
  const source = await readFile(inputPath);
  const info = pngLoad(source);

  console.log(`Processing: ${inputPath}`);
  console.log(`Original size: ${info.width}x${info.height}`);

  // Calculate resize dimensions (maintain aspect ratio)
  let targetWidth = info.width;
  let targetHeight = info.height;

  if (info.width > options.maxWidth) {
    targetWidth = options.maxWidth;
    targetHeight = Math.round(info.height * (options.maxWidth / info.width));
  }

  if (targetHeight > options.maxHeight) {
    targetHeight = options.maxHeight;
    targetWidth = Math.round(targetWidth * (options.maxHeight / targetHeight));
  }

  // Resize if needed
  let processed = source;
  if (targetWidth !== info.width || targetHeight !== info.height) {
    processed = resize(source, targetWidth, targetHeight, "Lanczos3");
    console.log(`Resized to: ${targetWidth}x${targetHeight}`);
  }

  // Generate output formats
  const baseName = inputPath.split("/").pop()?.replace(/\.[^.]+$/, "") ?? "image";

  if (options.outputFormats.includes("png")) {
    const optimized = pngOptimize(processed);
    await writeFile(`${outputDir}/${baseName}.png`, optimized);
    console.log(`PNG: ${optimized.byteLength} bytes`);
  }

  if (options.outputFormats.includes("webp")) {
    const webp = pngToWebp(processed, options.webpQuality);
    await writeFile(`${outputDir}/${baseName}.webp`, webp);
    console.log(`WebP: ${webp.byteLength} bytes`);
  }
}

// Usage
await processImage("./uploads/photo.png", "./processed", {
  maxWidth: 1920,
  maxHeight: 1080,
  webpQuality: 85,
  outputFormats: ["png", "webp"],
});
```

### App Icon Generator

```typescript
import {
  svgToPng,
  pngToIco,
  resize,
  pngOptimize,
  faviconCreate,
} from "runtime:image_tools";
import { readText, writeFile, mkdir } from "runtime:fs";

interface IconConfig {
  name: string;
  sizes: number[];
}

const PLATFORM_ICONS: Record<string, IconConfig[]> = {
  macos: [
    { name: "icon_16x16", sizes: [16, 32] },      // @1x, @2x
    { name: "icon_32x32", sizes: [32, 64] },
    { name: "icon_128x128", sizes: [128, 256] },
    { name: "icon_256x256", sizes: [256, 512] },
    { name: "icon_512x512", sizes: [512, 1024] },
  ],
  windows: [
    { name: "icon", sizes: [16, 32, 48, 64, 128, 256] },
  ],
  web: [
    { name: "favicon", sizes: [16, 32, 48, 180] },
  ],
};

async function generateAppIcons(
  svgPath: string,
  outputDir: string
): Promise<void> {
  const svg = await readText(svgPath);

  // macOS icons
  const macosDir = `${outputDir}/macos/AppIcon.iconset`;
  await mkdir(macosDir, { recursive: true });

  for (const config of PLATFORM_ICONS.macos) {
    for (let i = 0; i < config.sizes.length; i++) {
      const size = config.sizes[i];
      const suffix = i === 0 ? "" : "@2x";
      const png = svgToPng(svg, size, size);
      const optimized = pngOptimize(png);
      await writeFile(`${macosDir}/${config.name}${suffix}.png`, optimized);
    }
  }
  console.log("Generated macOS icons");

  // Windows ICO
  const windowsDir = `${outputDir}/windows`;
  await mkdir(windowsDir, { recursive: true });

  const windowsSizes = PLATFORM_ICONS.windows[0].sizes;
  const windowsPngs = windowsSizes.map(size => {
    const png = svgToPng(svg, size, size);
    return pngOptimize(png);
  });

  const ico = pngToIco(windowsPngs);
  await writeFile(`${windowsDir}/icon.ico`, ico);
  console.log("Generated Windows icon");

  // Web favicons
  const webDir = `${outputDir}/web`;
  await mkdir(webDir, { recursive: true });

  const largePng = svgToPng(svg, 512, 512);
  const favicons = faviconCreate(largePng);

  await writeFile(`${webDir}/favicon-16x16.png`, favicons.favicon16);
  await writeFile(`${webDir}/favicon-32x32.png`, favicons.favicon32);
  await writeFile(`${webDir}/favicon-48x48.png`, favicons.favicon48);
  await writeFile(`${webDir}/apple-touch-icon.png`, favicons.apple180);
  await writeFile(`${webDir}/favicon.ico`, favicons.ico);
  console.log("Generated web favicons");
}

// Usage
await generateAppIcons("./assets/logo.svg", "./icons");
```

### Image Gallery Processor

```typescript
import {
  pngInfo,
  resize,
  crop,
  pngOptimize,
  pngToWebp,
} from "runtime:image_tools";
import { readFile, writeFile, readDir } from "runtime:fs";

interface GalleryImage {
  original: string;
  thumbnail: string;
  medium: string;
  large: string;
}

async function processGallery(
  inputDir: string,
  outputDir: string
): Promise<GalleryImage[]> {
  const results: GalleryImage[] = [];

  const entries = await readDir(inputDir);

  for (const entry of entries) {
    if (!entry.name.endsWith(".png")) continue;

    const inputPath = `${inputDir}/${entry.name}`;
    const baseName = entry.name.replace(".png", "");

    console.log(`Processing: ${entry.name}`);

    const source = await readFile(inputPath);
    const info = pngInfo(source);

    // Create square thumbnail (crop center)
    const thumbSize = 150;
    const minDim = Math.min(info.width, info.height);
    const cropX = (info.width - minDim) / 2;
    const cropY = (info.height - minDim) / 2;

    const squared = crop(source, cropX, cropY, minDim, minDim);
    const thumbnail = resize(squared, thumbSize, thumbSize, "Lanczos3");
    const thumbOptimized = pngOptimize(thumbnail);

    await writeFile(`${outputDir}/${baseName}-thumb.png`, thumbOptimized);
    await writeFile(`${outputDir}/${baseName}-thumb.webp`, pngToWebp(thumbnail, 80));

    // Create medium (max 800px)
    const mediumMax = 800;
    let mediumWidth = info.width;
    let mediumHeight = info.height;

    if (Math.max(info.width, info.height) > mediumMax) {
      if (info.width > info.height) {
        mediumWidth = mediumMax;
        mediumHeight = Math.round(info.height * (mediumMax / info.width));
      } else {
        mediumHeight = mediumMax;
        mediumWidth = Math.round(info.width * (mediumMax / info.height));
      }
    }

    const medium = resize(source, mediumWidth, mediumHeight);
    await writeFile(`${outputDir}/${baseName}-medium.webp`, pngToWebp(medium, 85));

    // Create large (max 1920px)
    const largeMax = 1920;
    let largeWidth = info.width;
    let largeHeight = info.height;

    if (Math.max(info.width, info.height) > largeMax) {
      if (info.width > info.height) {
        largeWidth = largeMax;
        largeHeight = Math.round(info.height * (largeMax / info.width));
      } else {
        largeHeight = largeMax;
        largeWidth = Math.round(info.width * (largeMax / info.height));
      }
    }

    const large = resize(source, largeWidth, largeHeight);
    await writeFile(`${outputDir}/${baseName}-large.webp`, pngToWebp(large, 90));

    results.push({
      original: `${baseName}.png`,
      thumbnail: `${baseName}-thumb.webp`,
      medium: `${baseName}-medium.webp`,
      large: `${baseName}-large.webp`,
    });
  }

  return results;
}

// Usage
const gallery = await processGallery("./uploads", "./gallery");
console.log(`Processed ${gallery.length} images`);
```

## Best Practices

### Use Appropriate Filters

```typescript
// Pixel art - use Nearest to preserve sharp edges
const pixelArt = resize(source, 256, 256, "Nearest");

// Photos - use Lanczos3 for best quality
const photo = resize(source, 800, 600, "Lanczos3");

// Previews - use Bilinear for speed
const preview = resize(source, 100, 100, "Bilinear");
```

### Optimize Before Distribution

```typescript
import { pngOptimize, pngToWebp } from "runtime:image_tools";

// Always optimize PNGs to remove metadata and compress
const optimized = pngOptimize(pngData);

// Use WebP for web assets (better compression)
const webp = pngToWebp(pngData, 85);
```

### Handle Large Images

```typescript
// Process large images in steps
const source = await readFile("./large-image.png");
const info = pngInfo(source);

// Resize first if very large
if (info.width > 4000 || info.height > 4000) {
  const factor = 4000 / Math.max(info.width, info.height);
  source = scale(source, factor);
}

// Then apply other operations
const result = crop(source, x, y, w, h);
```
