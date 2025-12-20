---
title: "ext_svelte"
description: SvelteKit adapter extension providing the runtime:svelte module for SSR and ISR support.
slug: crates/ext-svelte
---

The `ext_svelte` crate provides SvelteKit integration for Forge applications through the `runtime:svelte` module.

## Overview

ext_svelte enables:

- **SvelteKit Adapter** - Build SvelteKit apps for Forge runtime
- **Server-Side Rendering** - Full SSR support within Forge
- **Incremental Static Regeneration** - ISR with configurable cache policies
- **Hybrid Rendering** - Mix of static, SSR, and ISR pages
- **Edge-like Caching** - In-memory and disk-based response caching

## Module: `runtime:svelte`

```typescript
import {
  serve,
  configure,
  invalidateCache,
  getCacheStats,
  prerender
} from "runtime:svelte";
```

## Key Types

### Configuration Types

```typescript
interface SvelteConfig {
  // Path to SvelteKit build output
  buildDir: string;

  // Server configuration
  server?: ServerConfig;

  // ISR/caching configuration
  isr?: IsrConfig;

  // Prerendering options
  prerender?: PrerenderConfig;
}

interface ServerConfig {
  // Port to listen on (if standalone)
  port?: number;

  // Hostname to bind
  host?: string;

  // Enable compression
  compress?: boolean;

  // CORS configuration
  cors?: CorsConfig;
}

interface IsrConfig {
  // Enable ISR
  enabled: boolean;

  // Default revalidation time in seconds
  defaultRevalidate?: number;

  // Maximum cache size in MB
  maxCacheSize?: number;

  // Cache storage: "memory" | "disk" | "hybrid"
  storage?: CacheStorage;

  // Path for disk cache
  cachePath?: string;
}

interface PrerenderConfig {
  // Routes to prerender at build time
  routes?: string[];

  // Crawl links from entry points
  crawl?: boolean;

  // Entry points for crawling
  entries?: string[];
}
```

### Cache Types

```typescript
interface CacheStats {
  // Number of cached entries
  entries: number;

  // Total cache size in bytes
  size: number;

  // Cache hit rate (0-1)
  hitRate: number;

  // Number of stale entries pending revalidation
  staleCount: number;
}

interface CacheEntry {
  // Route path
  path: string;

  // Creation timestamp
  createdAt: number;

  // Expiration timestamp
  expiresAt: number;

  // Whether entry is stale (serving while revalidating)
  stale: boolean;

  // Content hash
  etag: string;
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_svelte_serve` | `serve(config)` | Start SvelteKit server |
| `op_svelte_configure` | `configure(config)` | Update runtime configuration |
| `op_svelte_invalidate_cache` | `invalidateCache(patterns)` | Invalidate cached routes |
| `op_svelte_get_cache_stats` | `getCacheStats()` | Get cache statistics |
| `op_svelte_prerender` | `prerender(routes)` | Prerender specific routes |
| `op_svelte_handle_request` | `handleRequest(req)` | Handle incoming HTTP request |

## Usage Examples

### Basic Setup

```typescript
import { serve } from "runtime:svelte";

// Serve SvelteKit app with defaults
await serve({
  buildDir: "./build"
});
```

### With ISR Configuration

```typescript
import { serve, configure } from "runtime:svelte";

await serve({
  buildDir: "./build",
  isr: {
    enabled: true,
    defaultRevalidate: 60,  // Revalidate every 60 seconds
    storage: "hybrid",       // Memory + disk cache
    maxCacheSize: 100,       // 100MB max memory cache
    cachePath: "./.cache/svelte"
  }
});
```

### Cache Management

```typescript
import { invalidateCache, getCacheStats } from "runtime:svelte";

// Invalidate specific routes
await invalidateCache(["/blog/*", "/products/*"]);

// Invalidate everything
await invalidateCache(["*"]);

// Check cache status
const stats = await getCacheStats();
console.log(`Cache: ${stats.entries} entries, ${stats.hitRate * 100}% hit rate`);
```

### Prerendering

```typescript
import { prerender } from "runtime:svelte";

// Prerender specific high-traffic routes
await prerender([
  "/",
  "/about",
  "/products",
  "/blog"
]);
```

## SvelteKit Adapter Integration

To use with SvelteKit, install the Forge adapter:

```bash
npm install @anthropic/adapter-forge
```

Configure in `svelte.config.js`:

```javascript
import adapter from '@anthropic/adapter-forge';

export default {
  kit: {
    adapter: adapter({
      // ISR configuration per route
      isr: {
        '/blog/*': { revalidate: 300 },  // 5 minutes
        '/products/*': { revalidate: 60 }, // 1 minute
        '/static/*': false  // Fully static, no revalidation
      },

      // Routes to prerender at build time
      prerender: {
        entries: ['/', '/about'],
        crawl: true
      }
    })
  }
};
```

## ISR Behavior

### Stale-While-Revalidate

1. First request to a route - render and cache
2. Subsequent requests within `revalidate` period - serve from cache
3. Request after `revalidate` period:
   - Immediately serve stale cached version
   - Trigger background revalidation
   - Next request gets fresh content

### Cache Invalidation

Routes can be invalidated:
- Manually via `invalidateCache()`
- On webhook (e.g., CMS content update)
- Based on time (`revalidate` option)
- Memory pressure (LRU eviction)

## File Structure

```text
crates/ext_svelte/
├── src/
│   ├── lib.rs        # Extension implementation
│   ├── server.rs     # HTTP server integration
│   ├── isr.rs        # ISR/caching implementation
│   └── adapter.rs    # SvelteKit adapter logic
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Error Codes

```rust
enum SvelteErrorCode {
    Generic = 8600,
    BuildNotFound = 8601,
    RenderFailed = 8602,
    CacheError = 8603,
    ConfigInvalid = 8604,
    ServerError = 8605,
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `hyper` | HTTP server |
| `moka` | In-memory caching with TTL |
| `serde_json` | Configuration parsing |

## Related

- [runtime:net](/docs/crates/ext-net) - HTTP client operations
- [runtime:fs](/docs/crates/ext-fs) - File system for disk caching
- [SvelteKit Documentation](https://kit.svelte.dev/) - Official SvelteKit docs
