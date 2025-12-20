# Forge Performance Guide

Optimization strategies and best practices for building high-performance Forge applications.

## Table of Contents

1. [Extension Initialization](#extension-initialization)
2. [IPC Communication](#ipc-communication)
3. [Asset Serving](#asset-serving)
4. [Build Optimization](#build-optimization)
5. [Memory Management](#memory-management)
6. [JavaScript/TypeScript Performance](#javascripttypescript-performance)
7. [Window Rendering](#window-rendering)
8. [Profiling and Debugging](#profiling-and-debugging)

---

## Extension Initialization

### Tier System Performance Impact

Extensions initialize in four tiers based on complexity. This design optimizes startup time by:

1. **Parallel initialization within tiers** - Extensions in the same tier can initialize concurrently
2. **Dependency ordering** - Simpler extensions are ready before complex ones that depend on them
3. **Lazy state creation** - Only necessary state is initialized

#### Tier Breakdown

| Tier | Name | Init Time | Examples |
|------|------|-----------|----------|
| 0 | ExtensionOnly | ~0.1ms | ext_timers, ext_path |
| 1 | SimpleState | ~0.5ms | ext_fs, ext_net |
| 2 | CapabilityBased | ~1-2ms | Most extensions with permissions |
| 3 | ComplexContext | ~2-5ms | ext_ipc, ext_window |

**Total typical startup overhead:** ~50-100ms for all extensions

### Optimization Strategies

#### 1. Choose the Right Tier

When creating extensions, use the simplest tier possible:

```rust
// ✓ Good - Tier 0 (no state)
#[weld_op]
#[op2(fast)]
fn op_simple_math(#[number] a: i32, #[number] b: i32) -> i32 {
    a + b
}

// ✗ Avoid - Tier 2 when state isn't needed
// Don't create state just to cache simple calculations
```

#### 2. Lazy Initialization

Initialize expensive resources only when first used:

```rust
// State initialization
fn init_state(adapters: CapabilityAdapters) -> OpState {
    let state = OpState::new();
    // Don't create heavy resources here - wait for first use
    state.put(adapters);
    state
}

// First-use initialization in op
#[weld_op]
#[op2]
fn op_expensive_operation(state: &mut OpState) -> Result<String, Error> {
    // Check if heavy resource exists
    if !state.has::<HeavyResource>() {
        state.put(HeavyResource::new()?);
    }
    // Use resource...
}
```

#### 3. Avoid Synchronous Blocking

Never block the initialization thread:

```rust
// ✗ Bad - Blocks startup
fn init_state() -> OpState {
    let config = std::fs::read_to_string("config.json").unwrap(); // BLOCKS
    // ...
}

// ✓ Good - Load asynchronously after startup
fn init_state() -> OpState {
    // Initialize minimal state
    OpState::new()
}

#[weld_op(async)]
#[op2(async)]
async fn op_load_config() -> Result<Config, Error> {
    // Load config when actually needed
    load_config_async().await
}
```

---

## IPC Communication

### Understanding IPC Overhead

**Renderer → Deno:**
```text
window.host.send(channel, data)
  → Serialize (JSON) ~0.1-1ms
  → WebView IPC     ~0.5ms
  → Channel send    ~0.1ms
  → Deserialize     ~0.1-1ms
Total: ~1-3ms per message
```

**Deno → Renderer:**
```text
sendToWindow(windowId, channel, data)
  → Serialize (JSON) ~0.1-1ms
  → evaluate_script() ~1-2ms
  → Parse + dispatch  ~0.5ms
Total: ~2-4ms per message
```

### Optimization Strategies

#### 1. Batch Messages

**✗ Bad - Many small messages:**
```typescript
// Sends 100 separate IPC messages
for (let i = 0; i < 100; i++) {
    await updateProgress(i);
}
```

**✓ Good - Batch updates:**
```typescript
// Send one message per frame (60fps = ~16ms)
let pendingUpdates: number[] = [];

function queueUpdate(value: number) {
    pendingUpdates.push(value);
}

setInterval(() => {
    if (pendingUpdates.length > 0) {
        await updateProgressBatch(pendingUpdates);
        pendingUpdates = [];
    }
}, 16); // ~60fps
```

#### 2. Minimize Serialization Cost

**Serialization performance:**
- Small primitives: ~0.01ms
- Objects (10 fields): ~0.1ms
- Arrays (100 items): ~0.5ms
- Large JSON (1MB): ~10-50ms

**✗ Bad - Send entire large object:**
```typescript
// Serializes 1MB on every change
await sendConfig(entireConfigObject);
```

**✓ Good - Send only changes:**
```typescript
// Only serialize changed fields (~0.1ms)
await updateConfigField("theme", "dark");
```

#### 3. Use Binary for Large Data

For data >10KB, use `Uint8Array` instead of JSON:

```rust
// ✓ Good - Binary transfer (no JSON overhead)
#[weld_op]
#[op2]
fn op_read_image(#[string] path: String) -> Result<Vec<u8>, Error> {
    std::fs::read(path).map_err(|e| e.into())
}
```

```typescript
// Receive as Uint8Array (fast)
const imageData = await runtime.fs.readImage("./photo.jpg");
// imageData is Uint8Array - no parsing overhead
```

#### 4. Debounce High-Frequency Events

```typescript
// ✗ Bad - Sends 100+ messages/second
window.addEventListener("mousemove", (e) => {
    await sendMousePosition(e.clientX, e.clientY);
});

// ✓ Good - Throttle to 30fps
let lastSent = 0;
window.addEventListener("mousemove", (e) => {
    const now = Date.now();
    if (now - lastSent > 33) { // ~30fps
        sendMousePosition(e.clientX, e.clientY);
        lastSent = now;
    }
});
```

---

## Asset Serving

### Dev vs Production Performance

| Mode | Source | First Load | Reload | Cache |
|------|--------|------------|--------|-------|
| Dev | Filesystem | ~50ms | ~10ms | Browser cache |
| Production | Embedded binary | ~5ms | ~1ms | In-memory |

### Optimization: Embed Assets for Production

**In your extension's build.rs:**

```rust
fn main() {
    // Set FORGE_EMBED_DIR during release builds
    println!("cargo:rerun-if-env-changed=FORGE_EMBED_DIR");

    if std::env::var("PROFILE").unwrap() == "release" {
        println!("cargo:rustc-env=FORGE_EMBED_DIR=web");
    }
}
```

**Performance impact:**
- **Dev mode**: Assets loaded from disk on each request (~50ms cold, ~10ms warm)
- **Release mode**: Assets served from embedded bytes (~1-5ms)

### Asset Size Optimization

**1. Minify and compress:**
```bash
# In your app's build step
esbuild src/index.ts --bundle --minify --outfile=web/bundle.js

# Typical savings:
# - Development: 500KB
# - Production minified: 150KB (70% reduction)
# - Gzipped: 50KB (90% reduction)
```

**2. Code splitting:**
```typescript
// Split large features into separate chunks
const editor = await import('./editor.js'); // Loaded on demand
```

**3. Tree shaking:**
Ensure unused code is removed:
```typescript
// ✓ Good - Named imports (tree-shakeable)
import { readFile } from "runtime:fs";

// ✗ Bad - Imports entire namespace
import * as fs from "runtime:fs";
```

---

## Build Optimization

### esbuild Configuration

Forge uses esbuild for fast bundling. Optimize with these settings:

```javascript
// In your app's build.ts or build.js
import * as esbuild from "https://deno.land/x/esbuild/mod.js";

await esbuild.build({
    entryPoints: ["src/main.ts"],
    bundle: true,
    outfile: "web/bundle.js",

    // Performance optimizations
    minify: true,              // Reduce bundle size by ~70%
    treeShaking: true,         // Remove unused code
    target: "es2020",          // Modern JS (smaller output)
    platform: "browser",       // Browser-specific optimizations

    // Development speed
    sourcemap: "inline",       // Debug support (~20% overhead)

    // Advanced
    splitting: true,           // Code splitting (with format: "esm")
    format: "esm",            // Modern module format
    metafile: true,           // Analyze bundle composition
});
```

### Build Performance

| Configuration | Build Time | Bundle Size |
|--------------|------------|-------------|
| No optimization | ~50ms | 500KB |
| Minify only | ~150ms | 150KB |
| Full optimization | ~300ms | 100KB |

**Recommendation:** Use minimal optimization in dev, full in production.

---

## Memory Management

### Deno Heap Limits

Deno's V8 heap has default limits:
- **32-bit**: ~512MB
- **64-bit**: ~2GB (can grow to ~4GB)

### Monitoring Memory Usage

```typescript
// Check current memory usage
const memUsage = Deno.memoryUsage();
console.log("Heap used:", memUsage.heapUsed / 1024 / 1024, "MB");
console.log("Heap total:", memUsage.heapTotal / 1024 / 1024, "MB");
```

### Optimization Strategies

#### 1. Avoid Memory Leaks

**Common leak sources:**
```typescript
// ✗ Bad - Event listeners never removed
window.host.on("data", handleData);

// ✓ Good - Clean up listeners
const listener = window.host.on("data", handleData);
window.addEventListener("unload", () => {
    listener.remove();
});
```

#### 2. Release Large Buffers

```typescript
// ✗ Bad - Holds 10MB in memory indefinitely
const largeData = await fs.readBytes("large-file.bin");
processData(largeData);
// largeData still in memory...

// ✓ Good - Explicitly release
{
    const largeData = await fs.readBytes("large-file.bin");
    processData(largeData);
} // largeData eligible for GC here
```

#### 3. Stream Large Files

**✗ Bad - Loads entire file into memory:**
```typescript
const contents = await fs.readText("huge.log"); // 100MB in RAM
processLines(contents.split("\n"));
```

**✓ Good - Stream processing:**
```typescript
// Read in chunks (hypothetical streaming API)
for await (const chunk of fs.readStream("huge.log")) {
    processChunk(chunk);
} // Only ~64KB in memory at a time
```

#### 4. Use Weak References for Caches

```typescript
// Cache without preventing GC
const cache = new WeakMap<object, CachedData>();

function getCached(key: object): CachedData | undefined {
    return cache.get(key);
}
```

---

## JavaScript/TypeScript Performance

### Op Call Overhead

Each Rust op call has overhead:
- **Fast op** (`#[op2(fast)]`): ~0.01-0.05ms
- **Regular op**: ~0.1-0.5ms
- **Async op**: ~0.5-2ms (includes Promise overhead)

### Optimization: Minimize Op Calls

**✗ Bad - Many small calls:**
```typescript
// 100 separate op calls (~10ms total)
for (let i = 0; i < 100; i++) {
    await fs.writeText(`file${i}.txt`, data[i]);
}
```

**✓ Good - Batch operation:**
```typescript
// 1 op call (~0.5ms)
await fs.writeMultiple(files); // Single batched op
```

### Optimization: Use Fast Ops

```rust
// Fast path for synchronous operations
#[weld_op]
#[op2(fast)]
fn op_compute(#[number] a: i32, #[number] b: i32) -> i32 {
    a + b // No allocations, no Result
}
```

### TypeScript Optimization

**1. Avoid `any` type:**
```typescript
// ✗ Slower - Type checks at runtime
function process(data: any) {
    if (typeof data.value === "number") { /* ... */ }
}

// ✓ Faster - Type known at compile time
function process(data: { value: number }) {
    // No runtime checks needed
}
```

**2. Use const assertions:**
```typescript
// ✓ Type is literal ["read", "write"] not string[]
const operations = ["read", "write"] as const;
```

**3. Prefer for...of over forEach:**
```typescript
// ✓ Faster - No function call overhead per item
for (const item of items) {
    process(item);
}

// ✗ Slower - Function call per item
items.forEach(item => process(item));
```

---

## Window Rendering

### WebView Performance

The WebView is a full browser engine. Apply standard web performance practices:

#### 1. Minimize Reflows

```css
/* ✗ Bad - Causes multiple reflows */
element.style.width = "100px";
element.style.height = "100px";
element.style.margin = "10px";

/* ✓ Good - Single reflow */
element.style.cssText = "width: 100px; height: 100px; margin: 10px;";
```

#### 2. Use RequestAnimationFrame

```typescript
// ✗ Bad - Not synchronized with display
setInterval(() => {
    updateAnimation();
}, 16);

// ✓ Good - Synchronized with 60fps refresh
function animate() {
    updateAnimation();
    requestAnimationFrame(animate);
}
requestAnimationFrame(animate);
```

#### 3. Virtualize Long Lists

**✗ Bad - Render 10,000 items:**
```html
<div class="list">
    {items.map(item => <div>{item.name}</div>)}
</div>
```

**✓ Good - Render only visible items:**
```typescript
// Use virtual scrolling library
import VirtualList from "virtual-list-library";

<VirtualList
    items={items}
    itemHeight={40}
    renderItem={(item) => <div>{item.name}</div>}
/>
```

---

## Profiling and Debugging

### Deno Performance Profiling

#### 1. Built-in Profiler

```typescript
// Start profiling
Deno.core.ops.op_trace_start();

// Run code to profile
await heavyOperation();

// Stop and save profile
const profile = Deno.core.ops.op_trace_stop();
await Deno.writeTextFile("profile.json", JSON.stringify(profile));

// Load in Chrome DevTools: chrome://inspect → Load profile
```

#### 2. Measure Op Call Time

```typescript
console.time("operation");
await runtime.fs.readText("large-file.txt");
console.timeEnd("operation"); // Logs: operation: 45.2ms
```

#### 3. Memory Profiling

```typescript
// Take heap snapshot
const before = Deno.memoryUsage();

await potentiallyLeakyOperation();

const after = Deno.memoryUsage();
console.log("Heap growth:", (after.heapUsed - before.heapUsed) / 1024 / 1024, "MB");
```

### Rust Performance Profiling

#### 1. Criterion Benchmarks

```rust
// In benches/benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_op(c: &mut Criterion) {
    c.bench_function("op_example", |b| {
        b.iter(|| {
            op_example(black_box(100))
        });
    });
}

criterion_group!(benches, benchmark_op);
criterion_main!(benches);
```

Run with:
```bash
cargo bench
```

#### 2. Flamegraphs

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Profile your app
cargo flamegraph -p forge-runtime -- dev examples/my-app

# Opens flamegraph.svg in browser
```

#### 3. Release Mode Profiling

Always profile in release mode:
```bash
# Debug mode (10-100x slower)
cargo run -- dev examples/my-app

# Release mode (realistic performance)
cargo run --release -- dev examples/my-app
```

### Browser DevTools

WebView content can be debugged with browser DevTools:

**macOS:**
1. Enable Developer menu in Safari
2. Right-click in WebView → Inspect Element

**Windows:**
Set up edge://inspect for WebView2 debugging

---

## Performance Checklist

Use this checklist when optimizing:

### Startup Performance
- [ ] Extensions use appropriate tiers (simplest possible)
- [ ] Heavy initialization is lazy (first-use)
- [ ] No synchronous blocking in init
- [ ] Assets embedded for production builds

### Runtime Performance
- [ ] IPC messages batched (avoid >100/sec)
- [ ] Large data uses binary (Uint8Array) not JSON
- [ ] High-frequency events throttled/debounced
- [ ] Op calls minimized (batch when possible)

### Memory Performance
- [ ] Event listeners cleaned up on unload
- [ ] Large buffers released when done
- [ ] No accidental closures holding references
- [ ] Weak references used for caches

### Build Performance
- [ ] Production builds use minification
- [ ] Unused code tree-shaken
- [ ] Code splitting for large features
- [ ] Source maps only in dev mode

### Rendering Performance
- [ ] Long lists virtualized
- [ ] Animations use requestAnimationFrame
- [ ] Style changes batched
- [ ] Images lazy-loaded

---

## Performance Targets

Typical performance targets for Forge apps:

| Metric | Target | Good | Needs Improvement |
|--------|--------|------|-------------------|
| Startup time | <500ms | <300ms | >1s |
| IPC latency | <5ms | <2ms | >10ms |
| Frame rate | 60fps | 60fps | <30fps |
| Memory usage | <200MB | <100MB | >500MB |
| Bundle size | <500KB | <200KB | >1MB |
| Build time (dev) | <1s | <500ms | >3s |

---

## See Also

- [V8 Performance Tips](https://v8.dev/blog/performance)
- [Web Performance Best Practices](https://web.dev/fast/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Deno Performance](https://deno.land/manual/runtime/performance)
