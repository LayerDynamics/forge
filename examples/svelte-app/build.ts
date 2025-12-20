#!/usr/bin/env -S deno run --allow-read --allow-write --allow-env --allow-run --allow-net

/**
 * SvelteKit build script using Forge extensions
 * Uses ext_svelte for project detection, ext_log for structured logging,
 * and ext_trace for performance tracing.
 */

// Forge extensions
import { detect, walk, generateDeployConfig, generateSvelteData } from "runtime:svelte";
import { infoLog, debug, error, warn } from "runtime:log";
import { start as traceStart, end as traceEnd, instant } from "runtime:trace";

const isWatch = Deno.args.includes("--watch");

async function build() {
  const buildSpan = traceStart("build:svelte-app");
  instant("build:start", { mode: isWatch ? "watch" : "production" });

  try {
    // Step 1: Detect SvelteKit project
    infoLog("Detecting SvelteKit project...");
    const detection = await detect(".");

    debug("SvelteKit detection result", {
      isSveltekit: detection.isSveltekit,
      confidence: detection.confidence,
      hasSvelteConfig: detection.hasSvelteConfig,
      hasKitDependency: detection.hasKitDependency,
      hasRoutesDir: detection.hasRoutesDir,
      svelteVersion: detection.svelteVersion,
      kitVersion: detection.kitVersion,
      adapter: detection.adapter
    });

    if (detection.messages.length > 0) {
      for (const msg of detection.messages) {
        debug("Detection note", { message: msg });
      }
    }

    if (!detection.isSveltekit) {
      warn("Not a SvelteKit project", {
        confidence: detection.confidence,
        messages: detection.messages
      });
      // Fall back to custom esbuild build for raw Svelte
      await buildWithEsbuild();
      traceEnd(buildSpan, { success: true, method: "esbuild" });
      return;
    }

    infoLog("SvelteKit project detected", {
      svelteVersion: detection.svelteVersion,
      kitVersion: detection.kitVersion,
      confidence: detection.confidence,
      adapter: detection.adapter
    });

    // Step 2: Build with Vite
    if (isWatch) {
      infoLog("Starting Vite dev server...");
      const viteProcess = new Deno.Command("npm", {
        args: ["run", "dev"],
        stdout: "inherit",
        stderr: "inherit"
      });
      const child = viteProcess.spawn();
      await child.status;
    } else {
      infoLog("Running Vite build...");
      const viteBuildSpan = traceStart("vite:build");

      const viteProcess = new Deno.Command("npm", {
        args: ["run", "build"],
        stdout: "inherit",
        stderr: "inherit"
      });
      const { code } = await viteProcess.output();

      if (code !== 0) {
        traceEnd(viteBuildSpan, { success: false, exitCode: code });
        error("Vite build failed", { exitCode: code });
        Deno.exit(1);
      }

      traceEnd(viteBuildSpan, { success: true });

      // Step 3: Walk output directory and generate deploy config
      infoLog("Generating deploy configuration...");
      const files = await walk("web");
      debug("Found build files", { count: files.length });

      // Generate deploy config for static files
      const deployConfig = await generateDeployConfig(
        [], // prerendered pages (SPA mode = none)
        "web",
        "",
        files,
        "web"
      );

      // Generate svelte data (ISR configs - empty for SPA)
      const svelteData = generateSvelteData([]);

      debug("Deploy configuration generated", {
        staticFiles: deployConfig.staticFiles.length,
        redirects: deployConfig.redirects.length,
        headers: deployConfig.headers.length,
        isrConfigs: svelteData.isr.length
      });

      infoLog("Build complete!", {
        outputDir: "web",
        files: files.length
      });
    }

    traceEnd(buildSpan, { success: true, method: "vite" });
    instant("build:complete");
  } catch (err) {
    traceEnd(buildSpan, { success: false, error: String(err) });
    error("Build failed", { error: String(err) });
    Deno.exit(1);
  }
}

/**
 * Fallback build using esbuild for raw Svelte components
 * (Used when not a full SvelteKit project)
 */
async function buildWithEsbuild() {
  infoLog("Using esbuild fallback for raw Svelte build...");
  const esbuildSpan = traceStart("esbuild:build");

  try {
    // Dynamic import esbuild
    const esbuild = await import("https://deno.land/x/esbuild@v0.20.1/mod.js");
    const { compile } = await import("https://esm.sh/svelte@4.2.20/compiler");
    const ts = (await import("https://esm.sh/typescript@5.3.3")).default;

    // Custom Svelte plugin with TypeScript support
    function sveltePlugin(): esbuild.Plugin {
      return {
        name: "svelte",
        setup(build) {
          build.onLoad({ filter: /\.svelte$/ }, async (args) => {
            const source = await Deno.readTextFile(args.path);
            let processedSource = source;
            let preservedImports = "";

            const scriptMatch = source.match(/<script\s+lang=["']ts["'][^>]*>([\s\S]*?)<\/script>/);
            if (scriptMatch) {
              const tsCode = scriptMatch[1];
              const importRegex = /^\s*import\s+.+\s+from\s+['"][^'"]+['"];?\s*$/gm;
              const imports = (tsCode.match(importRegex) || []).map(s => s.trim());
              const svelteImports = imports.filter(imp => imp.includes('.svelte'));
              preservedImports = svelteImports.join('\n');

              const result = ts.transpileModule(tsCode, {
                compilerOptions: {
                  target: ts.ScriptTarget.ESNext,
                  module: ts.ModuleKind.ESNext,
                  verbatimModuleSyntax: false,
                },
              });
              processedSource = source.replace(
                /<script\s+lang=["']ts["']([^>]*)>/,
                "<script$1>"
              ).replace(tsCode, result.outputText);
            }

            const compiled = compile(processedSource, {
              filename: args.path,
              css: "injected",
              generate: "dom",
            });

            const finalCode = preservedImports + '\n' + compiled.js.code;

            return {
              contents: finalCode,
              loader: "js",
              warnings: compiled.warnings.map((w: { message: string; start?: { line: number; column: number } }) => ({
                text: w.message,
                location: w.start
                  ? { file: args.path, line: w.start.line, column: w.start.column }
                  : undefined,
              })),
            };
          });
        },
      };
    }

    // Plugin to resolve svelte imports
    function svelteResolvePlugin(): esbuild.Plugin {
      return {
        name: "svelte-resolve",
        setup(build) {
          build.onResolve({ filter: /^svelte(\/.*)?$/ }, (args) => {
            const path = args.path === "svelte"
              ? "https://esm.sh/svelte@4.2.20"
              : `https://esm.sh/svelte@4.2.20${args.path.slice(6)}`;
            return { path, namespace: "http-url" };
          });

          build.onResolve({ filter: /.*/, namespace: "http-url" }, (args) => {
            return { path: new URL(args.path, args.importer).href, namespace: "http-url" };
          });

          build.onLoad({ filter: /.*/, namespace: "http-url" }, async (args) => {
            const res = await fetch(args.path, { redirect: "follow" });
            if (!res.ok) throw new Error(`Failed to fetch ${args.path}: ${res.status}`);
            const contents = await res.text();
            return { contents, loader: "js" };
          });
        },
      };
    }

    const ctx = await esbuild.context({
      entryPoints: ["web/main.ts"],
      bundle: true,
      outfile: "web/bundle.js",
      format: "esm",
      sourcemap: true,
      plugins: [svelteResolvePlugin(), sveltePlugin()],
      loader: { ".ts": "ts" },
      logLevel: "info",
    });

    if (isWatch) {
      debug("Watching for changes with esbuild...");
      await ctx.watch();
    } else {
      await ctx.rebuild();
      await ctx.dispose();
      infoLog("Esbuild build complete!");
    }

    traceEnd(esbuildSpan, { success: true });
  } catch (err) {
    traceEnd(esbuildSpan, { success: false, error: String(err) });
    throw err;
  }
}

build().catch((err) => {
  error("Build failed", { error: String(err) });
  Deno.exit(1);
});
