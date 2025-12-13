// host:wasm module - JavaScript wrapper for Deno core ops
const core = Deno.core;

/**
 * Compile WASM bytes to a module
 * @param {Uint8Array} bytes - WASM binary
 * @returns {Promise<string>} Module ID
 */
export async function compile(bytes) {
  return await core.ops.op_wasm_compile(Array.from(bytes));
}

/**
 * Compile WASM from file
 * @param {string} path - Path to .wasm file
 * @returns {Promise<string>} Module ID
 */
export async function compileFile(path) {
  return await core.ops.op_wasm_compile_file(path);
}

/**
 * Drop a compiled module
 * @param {string} moduleId - Module ID to drop
 */
export async function dropModule(moduleId) {
  return await core.ops.op_wasm_drop_module(moduleId);
}

/**
 * @typedef {Object} WasiConfig
 * @property {Record<string, string>} [preopens] - Guest path -> Host path mappings
 * @property {Record<string, string>} [env] - Environment variables
 * @property {string[]} [args] - Command-line arguments
 * @property {boolean} [inheritStdin] - Inherit stdin from host
 * @property {boolean} [inheritStdout] - Inherit stdout from host
 * @property {boolean} [inheritStderr] - Inherit stderr from host
 */

/**
 * @typedef {Object} WasmInstance
 * @property {string} id - Instance ID
 * @property {string} moduleId - Module ID this instance was created from
 * @property {function(string, ...any): Promise<any[]>} call - Call an exported function
 * @property {function(): Promise<Array<{name: string, kind: string, params?: string[], results?: string[]}>>} getExports - Get list of exports
 * @property {Object} memory - Memory access object
 * @property {function(): Promise<void>} drop - Drop this instance
 */

/**
 * Instantiate a module
 * @param {string} moduleId - Module ID
 * @param {WasiConfig} [wasiConfig] - Optional WASI configuration
 * @returns {Promise<WasmInstance>} Instance handle object
 */
export async function instantiate(moduleId, wasiConfig) {
  // Convert camelCase to snake_case for Rust
  const config = wasiConfig ? {
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
     * @param {string} name - Function name
     * @param {...(number|{type: string, value: number})} args - Arguments
     * @returns {Promise<any[]>} Return values
     */
    async call(name, ...args) {
      // Normalize arguments to WasmValue format
      const normalizedArgs = args.map(arg => {
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
        return arg; // Already in {type, value} format
      });

      const results = await core.ops.op_wasm_call(instanceId, name, normalizedArgs);
      // Unwrap simple results
      return results.map(r => r.value);
    },

    /**
     * Get list of exports
     * @returns {Promise<Array<{name: string, kind: string, params?: string[], results?: string[]}>>}
     */
    async getExports() {
      return await core.ops.op_wasm_get_exports(instanceId);
    },

    /**
     * Memory access object
     */
    memory: {
      /**
       * Read bytes from memory
       * @param {number} offset - Byte offset
       * @param {number} length - Number of bytes
       * @returns {Promise<Uint8Array>}
       */
      async read(offset, length) {
        const bytes = await core.ops.op_wasm_memory_read(instanceId, offset, length);
        return new Uint8Array(bytes);
      },

      /**
       * Write bytes to memory
       * @param {number} offset - Byte offset
       * @param {Uint8Array} data - Data to write
       */
      async write(offset, data) {
        return await core.ops.op_wasm_memory_write(instanceId, offset, Array.from(data));
      },

      /**
       * Get memory size in pages
       * @returns {Promise<number>} Size in 64KB pages
       */
      async size() {
        return await core.ops.op_wasm_memory_size(instanceId);
      },

      /**
       * Grow memory
       * @param {number} pages - Number of pages to add
       * @returns {Promise<number>} Previous size in pages
       */
      async grow(pages) {
        return await core.ops.op_wasm_memory_grow(instanceId, pages);
      },
    },

    /**
     * Drop this instance
     */
    async drop() {
      return await core.ops.op_wasm_drop_instance(instanceId);
    },
  };
}

/**
 * Low-level function call (without instance wrapper)
 * @param {string} instanceId - Instance ID
 * @param {string} funcName - Function name
 * @param {Array<{type: string, value: number}>} args - Arguments in WasmValue format
 * @returns {Promise<Array<{type: string, value: number}>>} Return values
 */
export async function call(instanceId, funcName, args) {
  return await core.ops.op_wasm_call(instanceId, funcName, args);
}

/**
 * Low-level exports query
 * @param {string} instanceId - Instance ID
 * @returns {Promise<Array<{name: string, kind: string, params?: string[], results?: string[]}>>}
 */
export async function getExports(instanceId) {
  return await core.ops.op_wasm_get_exports(instanceId);
}

/**
 * Helper to create typed WASM values
 */
export const types = {
  /** Create i32 value */
  i32: (value) => ({ type: 'i32', value }),
  /** Create i64 value */
  i64: (value) => ({ type: 'i64', value: typeof value === 'bigint' ? Number(value) : value }),
  /** Create f32 value */
  f32: (value) => ({ type: 'f32', value }),
  /** Create f64 value */
  f64: (value) => ({ type: 'f64', value }),
};

/**
 * Memory operations namespace (low-level)
 */
export const memory = {
  async read(instanceId, offset, length) {
    const bytes = await core.ops.op_wasm_memory_read(instanceId, offset, length);
    return new Uint8Array(bytes);
  },
  async write(instanceId, offset, data) {
    return await core.ops.op_wasm_memory_write(instanceId, offset, Array.from(data));
  },
  async size(instanceId) {
    return await core.ops.op_wasm_memory_size(instanceId);
  },
  async grow(instanceId, pages) {
    return await core.ops.op_wasm_memory_grow(instanceId, pages);
  },
};
