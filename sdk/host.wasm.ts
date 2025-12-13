// host:wasm module - Deno API for WebAssembly loading and execution
// This is the single source of truth for the host:wasm SDK

// Type definitions

/** WASM value type */
export type WasmValueType = "i32" | "i64" | "f32" | "f64";

/** WASM value with explicit type */
export interface WasmValue {
  type: WasmValueType;
  value: number;
}

/** Export information from a WASM module */
export interface ExportInfo {
  name: string;
  kind: "function" | "memory" | "table" | "global";
  params?: WasmValueType[];
  results?: WasmValueType[];
}

/** WASI configuration for instantiating WASM modules */
export interface WasiConfig {
  /** Guest path to host path mappings for filesystem access */
  preopens?: Record<string, string>;
  /** Environment variables to expose to the WASM module */
  env?: Record<string, string>;
  /** Command-line arguments for the WASM module */
  args?: string[];
  /** Whether to inherit stdin from the host process */
  inheritStdin?: boolean;
  /** Whether to inherit stdout from the host process */
  inheritStdout?: boolean;
  /** Whether to inherit stderr from the host process */
  inheritStderr?: boolean;
}

/** Memory access interface for a WASM instance */
export interface WasmMemory {
  /**
   * Read bytes from WASM memory.
   * @param offset - Byte offset to start reading from
   * @param length - Number of bytes to read
   * @returns The bytes at the specified location
   */
  read(offset: number, length: number): Promise<Uint8Array>;

  /**
   * Write bytes to WASM memory.
   * @param offset - Byte offset to start writing at
   * @param data - The bytes to write
   */
  write(offset: number, data: Uint8Array): Promise<void>;

  /**
   * Get the current memory size in pages (1 page = 64KB).
   * @returns The memory size in pages
   */
  size(): Promise<number>;

  /**
   * Grow memory by the specified number of pages.
   * @param pages - Number of 64KB pages to add
   * @returns The previous memory size in pages
   */
  grow(pages: number): Promise<number>;
}

/** WASM instance handle with methods for calling functions and accessing memory */
export interface WasmInstance {
  /** The unique instance ID */
  readonly id: string;
  /** The module ID this instance was created from */
  readonly moduleId: string;

  /**
   * Call an exported WASM function.
   * @param name - The function name to call
   * @param args - Arguments (numbers are auto-typed, or use WasmValue for explicit types)
   * @returns Array of return values
   */
  call(name: string, ...args: (number | bigint | WasmValue)[]): Promise<number[]>;

  /**
   * Get list of all exports from this instance.
   * @returns Array of export information
   */
  getExports(): Promise<ExportInfo[]>;

  /** Memory access interface */
  memory: WasmMemory;

  /**
   * Drop this instance and free resources.
   */
  drop(): Promise<void>;
}

// Internal types for raw Rust responses
interface RawWasmValue {
  type: string;
  value: number;
}

interface RawExportInfo {
  name: string;
  kind: string;
  params?: string[];
  results?: string[];
}

// Deno.core.ops type declaration
declare const Deno: {
  core: {
    ops: {
      op_wasm_compile(bytes: number[]): Promise<string>;
      op_wasm_compile_file(path: string): Promise<string>;
      op_wasm_drop_module(moduleId: string): Promise<void>;
      op_wasm_instantiate(moduleId: string, wasiConfig?: object): Promise<string>;
      op_wasm_drop_instance(instanceId: string): Promise<void>;
      op_wasm_get_exports(instanceId: string): Promise<RawExportInfo[]>;
      op_wasm_call(instanceId: string, funcName: string, args: RawWasmValue[]): Promise<RawWasmValue[]>;
      op_wasm_memory_read(instanceId: string, offset: number, length: number): Promise<number[]>;
      op_wasm_memory_write(instanceId: string, offset: number, data: number[]): Promise<void>;
      op_wasm_memory_size(instanceId: string): Promise<number>;
      op_wasm_memory_grow(instanceId: string, pages: number): Promise<number>;
    };
  };
};

/**
 * Compile WASM bytes to a module.
 * Subject to manifest permissions (wasm.load).
 *
 * @param bytes - The WASM binary bytes
 * @returns A module ID that can be used to instantiate the module
 *
 * @example
 * ```ts
 * import { compile, instantiate } from "host:wasm";
 * import { readBytes } from "host:fs";
 *
 * const bytes = await readBytes("./module.wasm");
 * const moduleId = await compile(bytes);
 * const instance = await instantiate(moduleId);
 * ```
 */
export async function compile(bytes: Uint8Array): Promise<string> {
  return await Deno.core.ops.op_wasm_compile(Array.from(bytes));
}

/**
 * Compile WASM from a file path.
 * Subject to manifest permissions (wasm.load).
 *
 * @param path - Path to the .wasm file
 * @returns A module ID that can be used to instantiate the module
 *
 * @example
 * ```ts
 * import { compileFile, instantiate } from "host:wasm";
 *
 * const moduleId = await compileFile("./math.wasm");
 * const instance = await instantiate(moduleId);
 * const [result] = await instance.call("add", 2, 3);
 * console.log(result); // 5
 * ```
 */
export async function compileFile(path: string): Promise<string> {
  return await Deno.core.ops.op_wasm_compile_file(path);
}

/**
 * Drop a compiled module and free its resources.
 *
 * @param moduleId - The module ID to drop
 *
 * @example
 * ```ts
 * import { compileFile, dropModule } from "host:wasm";
 *
 * const moduleId = await compileFile("./module.wasm");
 * // Use the module...
 * await dropModule(moduleId);
 * ```
 */
export async function dropModule(moduleId: string): Promise<void> {
  return await Deno.core.ops.op_wasm_drop_module(moduleId);
}

/**
 * Instantiate a compiled WASM module.
 * Optionally configure WASI for modules that need system access.
 *
 * @param moduleId - The module ID from compile/compileFile
 * @param wasiConfig - Optional WASI configuration for system access
 * @returns A WasmInstance handle for calling functions
 *
 * @example
 * ```ts
 * import { compileFile, instantiate } from "host:wasm";
 *
 * // Basic instantiation
 * const moduleId = await compileFile("./pure.wasm");
 * const instance = await instantiate(moduleId);
 *
 * // With WASI configuration
 * const wasiModuleId = await compileFile("./app.wasm");
 * const wasiInstance = await instantiate(wasiModuleId, {
 *   preopens: { "/data": "./app-data" },
 *   env: { "HOME": "/data" },
 *   args: ["app", "--verbose"],
 *   inheritStdout: true,
 * });
 * await wasiInstance.call("_start");
 * ```
 */
export async function instantiate(moduleId: string, wasiConfig?: WasiConfig): Promise<WasmInstance> {
  // Convert camelCase to snake_case for Rust
  const config = wasiConfig ? {
    preopens: wasiConfig.preopens,
    env: wasiConfig.env,
    args: wasiConfig.args,
    inherit_stdin: wasiConfig.inheritStdin,
    inherit_stdout: wasiConfig.inheritStdout,
    inherit_stderr: wasiConfig.inheritStderr,
  } : undefined;

  const instanceId = await Deno.core.ops.op_wasm_instantiate(moduleId, config);

  return {
    id: instanceId,
    moduleId,

    async call(name: string, ...args: (number | bigint | WasmValue)[]): Promise<number[]> {
      // Normalize arguments to WasmValue format
      const normalizedArgs: RawWasmValue[] = args.map(arg => {
        if (typeof arg === "number") {
          // Default to i32 for integers, f64 for floats
          if (Number.isInteger(arg) && arg >= -2147483648 && arg <= 2147483647) {
            return { type: "i32", value: arg };
          } else if (Number.isInteger(arg)) {
            return { type: "i64", value: arg };
          } else {
            return { type: "f64", value: arg };
          }
        }
        if (typeof arg === "bigint") {
          return { type: "i64", value: Number(arg) };
        }
        return { type: arg.type, value: arg.value };
      });

      const results = await Deno.core.ops.op_wasm_call(instanceId, name, normalizedArgs);
      // Unwrap simple results
      return results.map(r => r.value);
    },

    async getExports(): Promise<ExportInfo[]> {
      const raw = await Deno.core.ops.op_wasm_get_exports(instanceId);
      return raw.map(e => ({
        name: e.name,
        kind: e.kind as ExportInfo["kind"],
        params: e.params as WasmValueType[] | undefined,
        results: e.results as WasmValueType[] | undefined,
      }));
    },

    memory: {
      async read(offset: number, length: number): Promise<Uint8Array> {
        const bytes = await Deno.core.ops.op_wasm_memory_read(instanceId, offset, length);
        return new Uint8Array(bytes);
      },

      async write(offset: number, data: Uint8Array): Promise<void> {
        return await Deno.core.ops.op_wasm_memory_write(instanceId, offset, Array.from(data));
      },

      async size(): Promise<number> {
        return await Deno.core.ops.op_wasm_memory_size(instanceId);
      },

      async grow(pages: number): Promise<number> {
        return await Deno.core.ops.op_wasm_memory_grow(instanceId, pages);
      },
    },

    async drop(): Promise<void> {
      return await Deno.core.ops.op_wasm_drop_instance(instanceId);
    },
  };
}

/**
 * Low-level function call without instance wrapper.
 *
 * @param instanceId - The instance ID
 * @param funcName - The function name to call
 * @param args - Arguments in WasmValue format
 * @returns Array of return values in WasmValue format
 */
export async function call(
  instanceId: string,
  funcName: string,
  args: WasmValue[]
): Promise<WasmValue[]> {
  const results = await Deno.core.ops.op_wasm_call(instanceId, funcName, args);
  return results.map(r => ({ type: r.type as WasmValueType, value: r.value }));
}

/**
 * Low-level exports query.
 *
 * @param instanceId - The instance ID
 * @returns Array of export information
 */
export async function getExports(instanceId: string): Promise<ExportInfo[]> {
  const raw = await Deno.core.ops.op_wasm_get_exports(instanceId);
  return raw.map(e => ({
    name: e.name,
    kind: e.kind as ExportInfo["kind"],
    params: e.params as WasmValueType[] | undefined,
    results: e.results as WasmValueType[] | undefined,
  }));
}

/**
 * Helper functions to create typed WASM values.
 *
 * @example
 * ```ts
 * import { instantiate, types } from "host:wasm";
 *
 * const [result] = await instance.call("process", types.i32(42), types.f64(3.14));
 * ```
 */
export const types = {
  /** Create an i32 (32-bit integer) value */
  i32: (value: number): WasmValue => ({ type: "i32", value }),
  /** Create an i64 (64-bit integer) value */
  i64: (value: number | bigint): WasmValue => ({
    type: "i64",
    value: typeof value === "bigint" ? Number(value) : value,
  }),
  /** Create an f32 (32-bit float) value */
  f32: (value: number): WasmValue => ({ type: "f32", value }),
  /** Create an f64 (64-bit float) value */
  f64: (value: number): WasmValue => ({ type: "f64", value }),
};

/**
 * Low-level memory operations namespace.
 */
export const memory = {
  /**
   * Read bytes from WASM instance memory.
   */
  async read(instanceId: string, offset: number, length: number): Promise<Uint8Array> {
    const bytes = await Deno.core.ops.op_wasm_memory_read(instanceId, offset, length);
    return new Uint8Array(bytes);
  },

  /**
   * Write bytes to WASM instance memory.
   */
  async write(instanceId: string, offset: number, data: Uint8Array): Promise<void> {
    return await Deno.core.ops.op_wasm_memory_write(instanceId, offset, Array.from(data));
  },

  /**
   * Get memory size in pages (1 page = 64KB).
   */
  async size(instanceId: string): Promise<number> {
    return await Deno.core.ops.op_wasm_memory_size(instanceId);
  },

  /**
   * Grow memory by pages.
   */
  async grow(instanceId: string, pages: number): Promise<number> {
    return await Deno.core.ops.op_wasm_memory_grow(instanceId, pages);
  },
};

// Re-export types for convenience
export type {
  WasmValueType,
  WasmValue,
  ExportInfo,
  WasiConfig,
  WasmMemory,
  WasmInstance,
};
