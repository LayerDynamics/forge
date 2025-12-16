// runtime:wasm module - TypeScript wrapper for Deno core ops

declare const Deno: {
  core: {
    ops: {
      op_wasm_compile(bytes: number[]): Promise<string>;
      op_wasm_compile_file(path: string): Promise<string>;
      op_wasm_drop_module(moduleId: string): Promise<void>;
      op_wasm_instantiate(moduleId: string, config: RawWasiConfig | undefined): Promise<string>;
      op_wasm_call(instanceId: string, name: string, args: WasmValue[]): Promise<WasmValue[]>;
      op_wasm_get_exports(instanceId: string): Promise<WasmExport[]>;
      op_wasm_memory_read(instanceId: string, offset: number, length: number): Promise<number[]>;
      op_wasm_memory_write(instanceId: string, offset: number, data: number[]): Promise<void>;
      op_wasm_memory_size(instanceId: string): Promise<number>;
      op_wasm_memory_grow(instanceId: string, pages: number): Promise<number>;
      op_wasm_drop_instance(instanceId: string): Promise<void>;
    };
  };
};

interface WasiConfig {
  preopens?: Record<string, string>;
  env?: Record<string, string>;
  args?: string[];
  inheritStdin?: boolean;
  inheritStdout?: boolean;
  inheritStderr?: boolean;
}

interface RawWasiConfig {
  preopens?: Record<string, string>;
  env?: Record<string, string>;
  args?: string[];
  inherit_stdin?: boolean;
  inherit_stdout?: boolean;
  inherit_stderr?: boolean;
}

interface WasmValue {
  type: "i32" | "i64" | "f32" | "f64";
  value: number;
}

interface WasmExport {
  name: string;
  kind: string;
  params?: string[];
  results?: string[];
}

interface MemoryAccess {
  read(offset: number, length: number): Promise<Uint8Array>;
  write(offset: number, data: Uint8Array): Promise<void>;
  size(): Promise<number>;
  grow(pages: number): Promise<number>;
}

interface WasmInstance {
  readonly id: string;
  readonly moduleId: string;
  call(name: string, ...args: (number | bigint | WasmValue)[]): Promise<number[]>;
  getExports(): Promise<WasmExport[]>;
  memory: MemoryAccess;
  drop(): Promise<void>;
}

const core = Deno.core;

/**
 * Compile WASM bytes to a module
 */
export async function compile(bytes: Uint8Array): Promise<string> {
  return await core.ops.op_wasm_compile(Array.from(bytes));
}

/**
 * Compile WASM from file
 */
export async function compileFile(path: string): Promise<string> {
  return await core.ops.op_wasm_compile_file(path);
}

/**
 * Drop a compiled module
 */
export async function dropModule(moduleId: string): Promise<void> {
  return await core.ops.op_wasm_drop_module(moduleId);
}

/**
 * Instantiate a module
 */
export async function instantiate(moduleId: string, wasiConfig?: WasiConfig): Promise<WasmInstance> {
  // Convert camelCase to snake_case for Rust
  const config: RawWasiConfig | undefined = wasiConfig ? {
    preopens: wasiConfig.preopens,
    env: wasiConfig.env,
    args: wasiConfig.args,
    inherit_stdin: wasiConfig.inheritStdin,
    inherit_stdout: wasiConfig.inheritStdout,
    inherit_stderr: wasiConfig.inheritStderr,
  } : undefined;

  const instanceId = await core.ops.op_wasm_instantiate(moduleId, config);

  return {
    id: instanceId,
    moduleId,

    /**
     * Call an exported function
     */
    async call(name: string, ...args: (number | bigint | WasmValue)[]): Promise<number[]> {
      // Normalize arguments to WasmValue format
      const normalizedArgs: WasmValue[] = args.map(arg => {
        if (typeof arg === 'number') {
          // Default to i32 for integers, f64 for floats
          if (Number.isInteger(arg) && arg >= -2147483648 && arg <= 2147483647) {
            return { type: 'i32', value: arg };
          } else if (Number.isInteger(arg)) {
            return { type: 'i64', value: arg };
          } else {
            return { type: 'f64', value: arg };
          }
        }
        if (typeof arg === 'bigint') {
          return { type: 'i64', value: Number(arg) };
        }
        return arg as WasmValue; // Already in {type, value} format
      });

      const results = await core.ops.op_wasm_call(instanceId, name, normalizedArgs);
      // Unwrap simple results
      return results.map(r => r.value);
    },

    /**
     * Get list of exports
     */
    async getExports(): Promise<WasmExport[]> {
      return await core.ops.op_wasm_get_exports(instanceId);
    },

    /**
     * Memory access object
     */
    memory: {
      /**
       * Read bytes from memory
       */
      async read(offset: number, length: number): Promise<Uint8Array> {
        const bytes = await core.ops.op_wasm_memory_read(instanceId, offset, length);
        return new Uint8Array(bytes);
      },

      /**
       * Write bytes to memory
       */
      async write(offset: number, data: Uint8Array): Promise<void> {
        return await core.ops.op_wasm_memory_write(instanceId, offset, Array.from(data));
      },

      /**
       * Get memory size in pages
       */
      async size(): Promise<number> {
        return await core.ops.op_wasm_memory_size(instanceId);
      },

      /**
       * Grow memory
       */
      async grow(pages: number): Promise<number> {
        return await core.ops.op_wasm_memory_grow(instanceId, pages);
      },
    },

    /**
     * Drop this instance
     */
    async drop(): Promise<void> {
      return await core.ops.op_wasm_drop_instance(instanceId);
    },
  };
}

/**
 * Low-level function call (without instance wrapper)
 */
export async function call(instanceId: string, funcName: string, args: WasmValue[]): Promise<WasmValue[]> {
  return await core.ops.op_wasm_call(instanceId, funcName, args);
}

/**
 * Low-level exports query
 */
export async function getExports(instanceId: string): Promise<WasmExport[]> {
  return await core.ops.op_wasm_get_exports(instanceId);
}

/**
 * Helper to create typed WASM values
 */
export const types = {
  /** Create i32 value */
  i32: (value: number): WasmValue => ({ type: 'i32', value }),
  /** Create i64 value */
  i64: (value: number | bigint): WasmValue => ({ type: 'i64', value: typeof value === 'bigint' ? Number(value) : value }),
  /** Create f32 value */
  f32: (value: number): WasmValue => ({ type: 'f32', value }),
  /** Create f64 value */
  f64: (value: number): WasmValue => ({ type: 'f64', value }),
};

/**
 * Memory operations namespace (low-level)
 */
export const memory = {
  async read(instanceId: string, offset: number, length: number): Promise<Uint8Array> {
    const bytes = await core.ops.op_wasm_memory_read(instanceId, offset, length);
    return new Uint8Array(bytes);
  },
  async write(instanceId: string, offset: number, data: Uint8Array): Promise<void> {
    return await core.ops.op_wasm_memory_write(instanceId, offset, Array.from(data));
  },
  async size(instanceId: string): Promise<number> {
    return await core.ops.op_wasm_memory_size(instanceId);
  },
  async grow(instanceId: string, pages: number): Promise<number> {
    return await core.ops.op_wasm_memory_grow(instanceId, pages);
  },
};
