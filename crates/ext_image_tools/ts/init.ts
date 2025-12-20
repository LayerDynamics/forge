// runtime:image_tools - Image manipulation and format conversion
//
// Provides general-purpose image operations:
// - PNG: load, save, info, optimize
// - SVG: load, info, render to raster
// - WebP: encode/decode for app asset optimization (NOT for icons)
// - Convert: SVG-to-PNG, PNG-to-ICO, favicons, PNG-to-WebP
// - Transform: resize, scale, crop, rotate, flip

declare const Deno: {
  core: {
    ops: {
      // PNG operations
      op_image_png_info(data: Uint8Array): ImageInfo;
      op_image_png_load(data: Uint8Array): ImageInfo;
      op_image_png_save(data: Uint8Array, options?: PngSaveOptions): Uint8Array;
      op_image_png_optimize(data: Uint8Array): Uint8Array;

      // SVG operations
      op_image_svg_info(svgData: string): SvgInfo;
      op_image_svg_load(svgData: string): SvgInfo;

      // WebP operations (for app asset optimization)
      op_image_webp_encode(data: Uint8Array, quality: number): Uint8Array;
      op_image_webp_decode(data: Uint8Array): Uint8Array;
      op_image_webp_info(data: Uint8Array): WebPInfo;

      // Convert operations
      op_image_svg_to_png(svgData: string, width: number, height: number): Uint8Array;
      op_image_png_to_ico(pngData: Uint8Array[]): Uint8Array;
      op_image_ico_extract(icoData: Uint8Array): Uint8Array[];
      op_image_favicon_create(pngData: Uint8Array): FaviconSet;
      op_image_png_to_webp(data: Uint8Array, quality: number): Uint8Array;

      // Transform operations
      op_image_resize(data: Uint8Array, width: number, height: number, filter?: FilterType): Uint8Array;
      op_image_scale(data: Uint8Array, factor: number): Uint8Array;
      op_image_crop(data: Uint8Array, x: number, y: number, width: number, height: number): Uint8Array;
      op_image_rotate(data: Uint8Array, degrees: number): Uint8Array;
      op_image_flip(data: Uint8Array, direction: FlipDirection): Uint8Array;
    };
  };
};

const { core } = Deno;

// ============================================================================
// Types
// ============================================================================

/** Information about an image */
export interface ImageInfo {
  width: number;
  height: number;
  format: string;
  hasAlpha: boolean;
  colorType: string;
}

/** Information about an SVG */
export interface SvgInfo {
  width: number;
  height: number;
  viewBox?: ViewBox;
}

/** SVG viewBox definition */
export interface ViewBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Information about a WebP image */
export interface WebPInfo {
  width: number;
  height: number;
  hasAlpha: boolean;
  isLossless: boolean;
}

/** Complete favicon set for web applications */
export interface FaviconSet {
  /** 16x16 favicon */
  favicon16: Uint8Array;
  /** 32x32 favicon */
  favicon32: Uint8Array;
  /** 48x48 favicon (high-DPI) */
  favicon48: Uint8Array;
  /** 180x180 Apple touch icon */
  apple180: Uint8Array;
  /** Multi-size ICO file */
  ico: Uint8Array;
}

/** PNG save options */
export interface PngSaveOptions {
  /** Compression level (0-9, default 6) */
  compression?: number;
}

/** Filter type for resizing */
export type FilterType = "Nearest" | "Bilinear" | "Lanczos3";

/** Direction for flipping */
export type FlipDirection = "Horizontal" | "Vertical";

// ============================================================================
// PNG Operations
// ============================================================================

/**
 * Get information about a PNG image
 * @param data - PNG image bytes
 * @returns Image information
 */
export function pngInfo(data: Uint8Array): ImageInfo {
  return core.ops.op_image_png_info(data);
}

/**
 * Load a PNG image and get its info
 * @param data - PNG image bytes
 * @returns Image information
 */
export function pngLoad(data: Uint8Array): ImageInfo {
  return core.ops.op_image_png_load(data);
}

/**
 * Save/re-encode a PNG image
 * @param data - Source image bytes
 * @param options - Optional save settings
 * @returns PNG bytes
 */
export function pngSave(data: Uint8Array, options?: PngSaveOptions): Uint8Array {
  return core.ops.op_image_png_save(data, options);
}

/**
 * Optimize a PNG by re-encoding (removes metadata, applies compression)
 * @param data - PNG image bytes
 * @returns Optimized PNG bytes
 */
export function pngOptimize(data: Uint8Array): Uint8Array {
  return core.ops.op_image_png_optimize(data);
}

// ============================================================================
// SVG Operations
// ============================================================================

/**
 * Get information about an SVG
 * @param svgData - SVG string content
 * @returns SVG information
 */
export function svgInfo(svgData: string): SvgInfo {
  return core.ops.op_image_svg_info(svgData);
}

/**
 * Load and parse an SVG
 * @param svgData - SVG string content
 * @returns SVG information
 */
export function svgLoad(svgData: string): SvgInfo {
  return core.ops.op_image_svg_load(svgData);
}

// ============================================================================
// WebP Operations (for app asset optimization only)
// ============================================================================

/**
 * Encode image as WebP (for app asset optimization)
 *
 * Note: WebP is intended for app asset optimization only,
 * NOT for icons or bundle-specific formats.
 *
 * @param data - Source image bytes (PNG)
 * @param quality - Quality level (0-100, 100 = lossless)
 * @returns WebP bytes
 */
export function webpEncode(data: Uint8Array, quality: number = 80): Uint8Array {
  return core.ops.op_image_webp_encode(data, quality);
}

/**
 * Decode WebP to PNG
 * @param data - WebP image bytes
 * @returns PNG bytes
 */
export function webpDecode(data: Uint8Array): Uint8Array {
  return core.ops.op_image_webp_decode(data);
}

/**
 * Get information about a WebP image
 * @param data - WebP image bytes
 * @returns WebP information
 */
export function webpInfo(data: Uint8Array): WebPInfo {
  return core.ops.op_image_webp_info(data);
}

// ============================================================================
// Conversion Operations
// ============================================================================

/**
 * Convert SVG to PNG at specified dimensions
 * @param svgData - SVG string content
 * @param width - Target width in pixels
 * @param height - Target height in pixels
 * @returns PNG bytes
 */
export function svgToPng(svgData: string, width: number, height: number): Uint8Array {
  return core.ops.op_image_svg_to_png(svgData, width, height);
}

/**
 * Convert PNG image(s) to ICO format
 *
 * If a single PNG is provided, it will be resized to standard ICO sizes.
 * If multiple PNGs are provided, they should be different sizes.
 *
 * @param pngData - Array of PNG image bytes
 * @returns ICO file bytes
 */
export function pngToIco(pngData: Uint8Array[]): Uint8Array {
  return core.ops.op_image_png_to_ico(pngData);
}

/**
 * Extract images from an ICO file
 * @param icoData - ICO file bytes
 * @returns Array of PNG bytes (one per size in the ICO)
 */
export function icoExtract(icoData: Uint8Array): Uint8Array[] {
  return core.ops.op_image_ico_extract(icoData);
}

/**
 * Create a complete favicon set from a source PNG
 *
 * Generates:
 * - 16x16, 32x32, 48x48 PNGs
 * - 180x180 Apple touch icon
 * - Multi-size ICO file
 *
 * @param pngData - Source PNG (should be at least 180x180, square)
 * @returns Complete favicon set
 */
export function faviconCreate(pngData: Uint8Array): FaviconSet {
  return core.ops.op_image_favicon_create(pngData);
}

/**
 * Convert PNG to WebP (for app asset optimization)
 * @param data - PNG image bytes
 * @param quality - Quality level (0-100, 100 = lossless)
 * @returns WebP bytes
 */
export function pngToWebp(data: Uint8Array, quality: number = 80): Uint8Array {
  return core.ops.op_image_png_to_webp(data, quality);
}

// ============================================================================
// Transform Operations
// ============================================================================

/**
 * Resize image to exact dimensions
 * @param data - Source image bytes
 * @param width - Target width in pixels
 * @param height - Target height in pixels
 * @param filter - Resize filter (default: Lanczos3)
 * @returns Resized PNG bytes
 */
export function resize(
  data: Uint8Array,
  width: number,
  height: number,
  filter: FilterType = "Lanczos3"
): Uint8Array {
  return core.ops.op_image_resize(data, width, height, filter);
}

/**
 * Scale image by a factor
 * @param data - Source image bytes
 * @param factor - Scale factor (e.g., 0.5 = half size, 2.0 = double)
 * @returns Scaled PNG bytes
 */
export function scale(data: Uint8Array, factor: number): Uint8Array {
  return core.ops.op_image_scale(data, factor);
}

/**
 * Crop a region from an image
 * @param data - Source image bytes
 * @param x - Left edge of crop region
 * @param y - Top edge of crop region
 * @param width - Width of crop region
 * @param height - Height of crop region
 * @returns Cropped PNG bytes
 */
export function crop(
  data: Uint8Array,
  x: number,
  y: number,
  width: number,
  height: number
): Uint8Array {
  return core.ops.op_image_crop(data, x, y, width, height);
}

/**
 * Rotate image by 90, 180, or 270 degrees
 * @param data - Source image bytes
 * @param degrees - Rotation angle (90, 180, or 270)
 * @returns Rotated PNG bytes
 */
export function rotate(data: Uint8Array, degrees: 90 | 180 | 270): Uint8Array {
  return core.ops.op_image_rotate(data, degrees);
}

/**
 * Flip image horizontally or vertically
 * @param data - Source image bytes
 * @param direction - Flip direction
 * @returns Flipped PNG bytes
 */
export function flip(data: Uint8Array, direction: FlipDirection): Uint8Array {
  return core.ops.op_image_flip(data, direction);
}
