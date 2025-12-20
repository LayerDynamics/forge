---
title: "ext_image_tools"
description: Image manipulation extension providing the runtime:image_tools module for PNG, SVG, WebP, and ICO operations.
slug: crates/ext-image-tools
---

The `ext_image_tools` crate provides image manipulation capabilities for Forge applications through the `runtime:image_tools` module.

## Overview

ext_image_tools handles:

- **Format Conversion** - Convert between PNG, WebP, SVG, ICO, JPEG
- **Resizing** - Scale images with various interpolation methods
- **Cropping** - Extract regions from images
- **Icon Generation** - Create multi-resolution ICO files and favicon sets
- **SVG Rasterization** - Render SVG to raster formats at any resolution
- **Optimization** - Compress images for web delivery

## Module: `runtime:image_tools`

```typescript
import {
  convert,
  resize,
  crop,
  generateIcons,
  generateFavicons,
  rasterizeSvg,
  optimize,
  getInfo
} from "runtime:image_tools";
```

## Key Types

### Configuration Types

```typescript
interface ConvertOptions {
  input: string;           // Input file path
  output: string;          // Output file path
  format?: ImageFormat;    // Target format (inferred from extension if omitted)
  quality?: number;        // Quality 1-100 (for lossy formats)
}

interface ResizeOptions {
  input: string;
  output: string;
  width?: number;          // Target width (maintains aspect if height omitted)
  height?: number;         // Target height (maintains aspect if width omitted)
  filter?: ResizeFilter;   // Interpolation method
  fit?: FitMode;           // How to fit image into dimensions
}

interface CropOptions {
  input: string;
  output: string;
  x: number;               // Left offset
  y: number;               // Top offset
  width: number;           // Crop width
  height: number;          // Crop height
}

interface IconSetOptions {
  input: string;           // Source image (should be square, >= 1024px)
  outputDir: string;       // Directory for generated icons
  sizes?: number[];        // Icon sizes to generate
  formats?: IconFormat[];  // Output formats
}
```

### Enums

```typescript
type ImageFormat = "png" | "webp" | "jpeg" | "svg" | "ico" | "gif";

type ResizeFilter =
  | "nearest"      // Fastest, pixelated
  | "triangle"     // Linear interpolation
  | "catmullRom"   // Good balance
  | "gaussian"     // Smooth
  | "lanczos3";    // Best quality, slowest

type FitMode =
  | "contain"      // Fit within dimensions, preserve aspect
  | "cover"        // Fill dimensions, crop overflow
  | "fill"         // Stretch to exact dimensions
  | "inside"       // Like contain, never upscale
  | "outside";     // Like cover, never downscale

type IconFormat = "ico" | "icns" | "png";
```

### Result Types

```typescript
interface ImageInfo {
  width: number;
  height: number;
  format: ImageFormat;
  hasAlpha: boolean;
  fileSize: number;
  colorDepth: number;
}

interface IconSetResult {
  files: GeneratedFile[];
  manifest?: string;       // Path to manifest.json for web icons
}

interface GeneratedFile {
  path: string;
  size: number;            // File size in bytes
  dimensions: [number, number];
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_image_convert` | `convert(options)` | Convert image format |
| `op_image_resize` | `resize(options)` | Resize image |
| `op_image_crop` | `crop(options)` | Crop image region |
| `op_image_generate_icons` | `generateIcons(options)` | Generate icon set |
| `op_image_generate_favicons` | `generateFavicons(options)` | Generate web favicons |
| `op_image_rasterize_svg` | `rasterizeSvg(options)` | Render SVG to raster |
| `op_image_optimize` | `optimize(options)` | Optimize/compress image |
| `op_image_get_info` | `getInfo(path)` | Get image metadata |

## Usage Examples

### Convert Format

```typescript
import { convert } from "runtime:image_tools";

// Convert PNG to WebP with quality setting
await convert({
  input: "./image.png",
  output: "./image.webp",
  format: "webp",
  quality: 85
});
```

### Resize Image

```typescript
import { resize } from "runtime:image_tools";

// Resize maintaining aspect ratio
await resize({
  input: "./photo.jpg",
  output: "./thumbnail.jpg",
  width: 200,
  filter: "lanczos3",
  fit: "contain"
});

// Exact dimensions with cover fit
await resize({
  input: "./photo.jpg",
  output: "./square.jpg",
  width: 500,
  height: 500,
  fit: "cover"
});
```

### Generate App Icons

```typescript
import { generateIcons } from "runtime:image_tools";

// Generate complete icon set for app bundling
const result = await generateIcons({
  input: "./assets/icon-1024.png",
  outputDir: "./build/icons",
  sizes: [16, 32, 64, 128, 256, 512, 1024],
  formats: ["ico", "icns", "png"]
});

console.log(`Generated ${result.files.length} icon files`);
```

### Generate Web Favicons

```typescript
import { generateFavicons } from "runtime:image_tools";

// Generate favicons for web deployment
const result = await generateFavicons({
  input: "./assets/logo.svg",
  outputDir: "./web/public",
  sizes: [16, 32, 180, 192, 512],
  generateManifest: true
});

// result.manifest contains path to manifest.json
```

### Rasterize SVG

```typescript
import { rasterizeSvg } from "runtime:image_tools";

// Render SVG at high resolution
await rasterizeSvg({
  input: "./vector.svg",
  output: "./vector@2x.png",
  width: 2048,
  height: 2048,
  background: "#ffffff"  // Optional background color
});
```

### Get Image Info

```typescript
import { getInfo } from "runtime:image_tools";

const info = await getInfo("./image.png");
console.log(`${info.width}x${info.height} ${info.format}`);
console.log(`Alpha: ${info.hasAlpha}, Size: ${info.fileSize} bytes`);
```

## File Structure

```text
crates/ext_image_tools/
├── src/
│   ├── lib.rs        # Extension implementation
│   ├── convert.rs    # Format conversion
│   ├── resize.rs     # Resize/scale operations
│   ├── icons.rs      # Icon set generation
│   └── svg.rs        # SVG rasterization
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `image` | Core image processing |
| `resvg` | SVG rasterization |
| `ico` | ICO file generation |
| `webp` | WebP encoding/decoding |
| `oxipng` | PNG optimization |

## Error Codes

```rust
enum ImageToolsErrorCode {
    Generic = 8400,
    FileNotFound = 8401,
    InvalidFormat = 8402,
    EncodingFailed = 8403,
    DecodingFailed = 8404,
    InvalidDimensions = 8405,
    IoError = 8406,
}
```

## Related

- [runtime:fs](/docs/crates/ext-fs) - File system operations
- [runtime:dock](/docs/crates/ext-dock) - Set custom dock icons (macOS)
- [forge_cli bundler](/docs/crates/forge-cli) - Uses icons during app bundling
