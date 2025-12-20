#!/usr/bin/env bash
# Generate Rust API documentation for Forge workspace
# This only documents workspace crates, not dependencies

set -e

echo "Generating Rust documentation..."
echo "================================"
echo

# Clean old docs
if [ -d "target/doc" ]; then
    echo "Cleaning old documentation..."
    rm -rf target/doc
fi

# Generate docs without dependencies
echo "Building documentation (excluding dependencies)..."
cargo doc --workspace --no-deps --document-private-items

# Create root index.html
echo "Creating workspace index page..."
cat > target/doc/index.html <<'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Forge Framework Documentation</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
            padding: 2rem;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 3rem;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        h1 {
            color: #2c3e50;
            margin-bottom: 0.5rem;
            font-size: 2.5rem;
        }
        .subtitle {
            color: #7f8c8d;
            margin-bottom: 2rem;
            font-size: 1.1rem;
        }
        .section {
            margin-top: 2.5rem;
        }
        .section h2 {
            color: #34495e;
            border-bottom: 2px solid #3498db;
            padding-bottom: 0.5rem;
            margin-bottom: 1rem;
        }
        .crate-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
            gap: 1rem;
            margin-top: 1rem;
        }
        .crate-card {
            border: 1px solid #e1e4e8;
            border-radius: 6px;
            padding: 1rem;
            background: #fafbfc;
            transition: all 0.2s;
        }
        .crate-card:hover {
            border-color: #3498db;
            box-shadow: 0 2px 8px rgba(52, 152, 219, 0.2);
            transform: translateY(-2px);
        }
        .crate-card a {
            text-decoration: none;
            color: #3498db;
            font-weight: 600;
            font-size: 1.1rem;
        }
        .crate-card a:hover {
            text-decoration: underline;
        }
        .crate-desc {
            color: #586069;
            font-size: 0.9rem;
            margin-top: 0.5rem;
        }
        .note {
            background: #fff3cd;
            border-left: 4px solid #ffc107;
            padding: 1rem;
            margin-top: 2rem;
            border-radius: 4px;
        }
        .note strong {
            color: #856404;
        }
        code {
            background: #f6f8fa;
            padding: 0.2rem 0.4rem;
            border-radius: 3px;
            font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
            font-size: 0.9em;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔨 Forge Framework</h1>
        <p class="subtitle">Rust API Documentation</p>

        <div class="section">
            <h2>Core Crates</h2>
            <div class="crate-grid">
                <div class="crate-card">
                    <a href="forge_cli/index.html">forge_cli</a>
                    <p class="crate-desc">Command-line interface for Forge (dev, build, bundle, docs)</p>
                </div>
                <div class="crate-card">
                    <a href="forge_runtime/index.html">forge-runtime</a>
                    <p class="crate-desc">Main runtime binary with Deno integration and window management</p>
                </div>
                <div class="crate-card">
                    <a href="forge_weld/index.html">forge-weld</a>
                    <p class="crate-desc">Code generation framework for TypeScript bindings</p>
                </div>
                <div class="crate-card">
                    <a href="forge_weld_macro/index.html">forge-weld-macro</a>
                    <p class="crate-desc">Proc macros: #[weld_op], #[weld_struct], #[weld_enum]</p>
                </div>
                <div class="crate-card">
                    <a href="forge_etch/index.html">forge-etch</a>
                    <p class="crate-desc">Documentation generator (Astro/HTML)</p>
                </div>
            </div>
        </div>

        <div class="section">
            <h2>Runtime Extensions (runtime:* modules)</h2>
            <div class="crate-grid">
                <div class="crate-card">
                    <a href="ext_fs/index.html">ext_fs</a>
                    <p class="crate-desc">File system operations (runtime:fs)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_window/index.html">ext_window</a>
                    <p class="crate-desc">Window management, menus, trays (runtime:window)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_ipc/index.html">ext_ipc</a>
                    <p class="crate-desc">Deno ↔ Renderer communication (runtime:ipc)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_net/index.html">ext_net</a>
                    <p class="crate-desc">HTTP fetch, network operations (runtime:net)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_sys/index.html">ext_sys</a>
                    <p class="crate-desc">System info, clipboard, notifications (runtime:sys)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_process/index.html">ext_process</a>
                    <p class="crate-desc">Spawn child processes (runtime:process)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_app/index.html">ext_app</a>
                    <p class="crate-desc">App lifecycle and info (runtime:app)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_crypto/index.html">ext_crypto</a>
                    <p class="crate-desc">Cryptographic operations (runtime:crypto)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_storage/index.html">ext_storage</a>
                    <p class="crate-desc">Persistent key-value storage (runtime:storage)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_database/index.html">ext_database</a>
                    <p class="crate-desc">Database operations (runtime:database)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_webview/index.html">ext_webview</a>
                    <p class="crate-desc">WebView management (runtime:webview)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_shell/index.html">ext_shell</a>
                    <p class="crate-desc">Shell command execution (runtime:shell)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_wasm/index.html">ext_wasm</a>
                    <p class="crate-desc">WebAssembly module loading (runtime:wasm)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_devtools/index.html">ext_devtools</a>
                    <p class="crate-desc">Developer tools integration (runtime:devtools)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_timers/index.html">ext_timers</a>
                    <p class="crate-desc">Timer operations (runtime:timers)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_shortcuts/index.html">ext_shortcuts</a>
                    <p class="crate-desc">Keyboard shortcuts (runtime:shortcuts)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_signals/index.html">ext_signals</a>
                    <p class="crate-desc">Signal handling (runtime:signals)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_updater/index.html">ext_updater</a>
                    <p class="crate-desc">App update management (runtime:updater)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_monitor/index.html">ext_monitor</a>
                    <p class="crate-desc">Display monitor info (runtime:monitor)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_display/index.html">ext_display</a>
                    <p class="crate-desc">Display management (runtime:display)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_log/index.html">ext_log</a>
                    <p class="crate-desc">Logging utilities (runtime:log)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_trace/index.html">ext_trace</a>
                    <p class="crate-desc">Tracing and diagnostics (runtime:trace)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_lock/index.html">ext_lock</a>
                    <p class="crate-desc">Lock file management (runtime:lock)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_path/index.html">ext_path</a>
                    <p class="crate-desc">Path utilities (runtime:path)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_protocol/index.html">ext_protocol</a>
                    <p class="crate-desc">Custom protocol handling (runtime:protocol)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_os_compat/index.html">ext_os_compat</a>
                    <p class="crate-desc">OS compatibility layer (runtime:os_compat)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_debugger/index.html">ext_debugger</a>
                    <p class="crate-desc">Debugger integration (runtime:debugger)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_bundler/index.html">ext_bundler</a>
                    <p class="crate-desc">Asset bundling (forge:bundler)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_codesign/index.html">ext_codesign</a>
                    <p class="crate-desc">Code signing utilities (forge:codesign)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_encoding/index.html">ext_encoding</a>
                    <p class="crate-desc">Encoding/decoding operations</p>
                </div>
                <div class="crate-card">
                    <a href="ext_etcher/index.html">ext_etcher</a>
                    <p class="crate-desc">Documentation etching (forge:etcher)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_svelte/index.html">ext_svelte</a>
                    <p class="crate-desc">Svelte integration (forge:svelte)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_weld/index.html">ext_weld</a>
                    <p class="crate-desc">Welding operations (forge:weld)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_dock/index.html">ext_dock</a>
                    <p class="crate-desc">macOS dock integration (runtime:dock)</p>
                </div>
                <div class="crate-card">
                    <a href="ext_image_tools/index.html">ext_image_tools</a>
                    <p class="crate-desc">Image processing utilities</p>
                </div>
                <div class="crate-card">
                    <a href="ext_web_inspector/index.html">ext_web_inspector</a>
                    <p class="crate-desc">Web inspector integration</p>
                </div>
            </div>
        </div>

        <div class="note">
            <strong>Note:</strong> This documentation covers only Forge workspace crates.
            Dependency documentation is excluded to keep the docs focused and manageable.
            Generated with <code>cargo doc --workspace --no-deps --document-private-items</code>
        </div>
    </div>
</body>
</html>
EOF

echo
echo "✓ Documentation generated successfully!"
echo "  Location: target/doc/"
echo "  Root index: target/doc/index.html"
echo
echo "To view in browser:"
echo "  open target/doc/index.html"
echo "  # or"
echo "  cargo doc --workspace --no-deps --open"
