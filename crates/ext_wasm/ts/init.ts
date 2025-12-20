/**
 * @module runtime:wasm
 *
 * WebAssembly (WASM) module loading and execution extension for Forge runtime.
 *
 * Provides comprehensive WebAssembly support including module compilation, instantiation,
 * function calls, linear memory access, and WASI (WebAssembly System Interface) integration
 * with capability-based security.
 *
 * ## Features
 *
 * ### Module Management
 * - Compile WASM modules from bytes or files
 * - Cache compiled modules for reuse across multiple instances
 * - Drop modules to free resources
 * - Automatic validation and error reporting
 *
 * ### Instance Management
 * - Instantiate modules with optional WASI configuration
 * - Multiple instances from single compiled module
 * - Instance-level resource management
 * - Automatic cleanup on drop
 *
 * ### Function Calls
 * - Call exported WASM functions with type-safe arguments
 * - Automatic type conversion (i32, i64, f32, f64)
 * - Support for multiple return values
 * - Introspect available exports
 *
 * ### Linear Memory Access
 * - Read and write memory at arbitrary offsets
 * - Page-based memory sizing (64KB pages)
 * - Dynamic memory growth
 * - Efficient Uint8Array integration
 *
 * ### WASI Support
 * - File system access with directory preopens
 * - Environment variables
 * - Command-line arguments
 * - Standard I/O inheritance (stdin, stdout, stderr)
 * - Capability-based security model
 *
 * ## Error Codes (5000-5011)
 *
 * | Code | Error | Description |
 * |------|-------|-------------|
 * | 5000 | CompileError | Failed to compile WASM module |
 * | 5001 | InstantiateError | Failed to instantiate module |
 * | 5002 | CallError | Function call failed |
 * | 5003 | ExportNotFound | Export not found in module |
 * | 5004 | InvalidModuleHandle | Invalid module handle |
 * | 5005 | InvalidInstanceHandle | Invalid instance handle |
 * | 5006 | MemoryError | Memory access error |
 * | 5007 | TypeError | Type mismatch in function call |
 * | 5008 | IoError | IO error (file loading) |
 * | 5009 | PermissionDenied | Permission denied by capability system |
 * | 5010 | WasiError | WASI configuration error |
 * | 5011 | FuelExhausted | Fuel exhaustion (execution limit) |
 *
 * ## WASM Value Types
 *
 * WebAssembly supports four numeric types:
 * - `i32`: 32-bit integer (-2,147,483,648 to 2,147,483,647)
 * - `i64`: 64-bit integer (larger range, use BigInt in JS)
 * - `f32`: 32-bit floating point (single precision)
 * - `f64`: 64-bit floating point (double precision)
 *
 * ## Memory Model
 *
 * WASM linear memory is organized in 64KB pages:
 * - Initial size specified in WASM module
 * - Can grow dynamically (up to maximum)
 * - Accessed as byte array from JavaScript
 * - Shared between module and host
 *
 * @example
 * ```typescript
 * import { compile, instantiate, types } from "runtime:wasm";
 *
 * // Compile WASM module
 * const wasmBytes = await Deno.readFile("module.wasm");
 * const moduleId = await compile(wasmBytes);
 *
 * // Instantiate with WASI
 * const instance = await instantiate(moduleId, {
 *   preopens: { "/data": "./data" },
 *   env: { "LOG_LEVEL": "debug" },
 *   args: ["--verbose"],
 *   inheritStdout: true
 * });
 *
 * // Call exported function
 * const result = await instance.call("add", types.i32(10), types.i32(32));
 * console.log("Result:", result[0]); // 42
 *
 * // Access memory
 * const data = await instance.memory.read(0, 100);
 * console.log("Memory:", data);
 *
 * // Cleanup
 * await instance.drop();
 * ```
 */

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

/**
 * WASI (WebAssembly System Interface) configuration.
 *
 * Configures the WASI environment for a WebAssembly instance, controlling
 * file system access, environment variables, command-line arguments, and
 * standard I/O behavior.
 *
 * @example
 * ```typescript
 * const config: WasiConfig = {
 *   // Map guest paths to host directories
 *   preopens: {
 *     "/data": "./app-data",
 *     "/config": "/etc/myapp"
 *   },
 *   // Environment variables
 *   env: {
 *     "LOG_LEVEL": "debug",
 *     "API_KEY": "secret"
 *   },
 *   // Command-line arguments
 *   args: ["--verbose", "--port", "3000"],
 *   // Inherit host's standard I/O
 *   inheritStdout: true,
 *   inheritStderr: true
 * };
 * ```
 */
interface WasiConfig {
  /** Map guest virtual paths to host directory paths (e.g., {"/data": "./app-data"}) */
  preopens?: Record<string, string>;
  /** Environment variables visible to WASM module */
  env?: Record<string, string>;
  /** Command-line arguments passed to WASM module */
  args?: string[];
  /** Inherit stdin from host process (default: false) */
  inheritStdin?: boolean;
  /** Inherit stdout from host process (default: false) */
  inheritStdout?: boolean;
  /** Inherit stderr from host process (default: false) */
  inheritStderr?: boolean;
}

/**
 * Internal WASI configuration (snake_case for Rust interop).
 *
 * @internal This is automatically converted from WasiConfig.
 */
interface RawWasiConfig {
  preopens?: Record<string, string>;
  env?: Record<string, string>;
  args?: string[];
  inherit_stdin?: boolean;
  inherit_stdout?: boolean;
  inherit_stderr?: boolean;
}

/**
 * Typed WebAssembly value.
 *
 * Represents a WebAssembly value with explicit type information. WASM supports
 * four numeric types: i32, i64, f32, f64.
 *
 * **Type Ranges:**
 * - `i32`: -2,147,483,648 to 2,147,483,647 (32-bit signed integer)
 * - `i64`: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 (64-bit signed integer)
 * - `f32`: Single-precision floating point (32-bit IEEE 754)
 * - `f64`: Double-precision floating point (64-bit IEEE 754)
 *
 * @example
 * ```typescript
 * // Manual value creation
 * const val1: WasmValue = { type: "i32", value: 42 };
 * const val2: WasmValue = { type: "f64", value: 3.14159 };
 *
 * // Or use helper functions
 * import { types } from "runtime:wasm";
 * const val3 = types.i32(42);
 * const val4 = types.f64(3.14159);
 * ```
 */
interface WasmValue {
  /** Value type (i32, i64, f32, or f64) */
  type: "i32" | "i64" | "f32" | "f64";
  /** Numeric value */
  value: number;
}

/**
 * WebAssembly module export metadata.
 *
 * Describes an exported item from a WASM module (function, memory, table, or global).
 *
 * **Export Kinds:**
 * - `"func"`: Exported function
 * - `"memory"`: Exported linear memory
 * - `"table"`: Exported table
 * - `"global"`: Exported global variable
 *
 * @example
 * ```typescript
 * const exports = await instance.getExports();
 * for (const exp of exports) {
 *   console.log(`${exp.name}: ${exp.kind}`);
 *   if (exp.kind === "func") {
 *     console.log(`  Params: ${exp.params?.join(", ")}`);
 *     console.log(`  Results: ${exp.results?.join(", ")}`);
 *   }
 * }
 * ```
 */
interface WasmExport {
  /** Export name (e.g., "add", "memory", "_start") */
  name: string;
  /** Export kind (func, memory, table, global) */
  kind: string;
  /** Function parameter types (only for functions) */
  params?: string[];
  /** Function return types (only for functions) */
  results?: string[];
}

/**
 * WebAssembly linear memory access interface.
 *
 * Provides methods to read, write, and manage WebAssembly linear memory.
 * Memory is organized in 64KB pages and can grow dynamically.
 *
 * **Page Size:** 64KB (65,536 bytes)
 *
 * @example
 * ```typescript
 * // Read 100 bytes from offset 0
 * const data = await instance.memory.read(0, 100);
 * console.log("First byte:", data[0]);
 *
 * // Write data to memory
 * const newData = new Uint8Array([1, 2, 3, 4, 5]);
 * await instance.memory.write(1000, newData);
 *
 * // Check memory size (in pages)
 * const pages = await instance.memory.size();
 * console.log(`Memory: ${pages} pages (${pages * 64} KB)`);
 *
 * // Grow memory by 10 pages (640KB)
 * const oldSize = await instance.memory.grow(10);
 * console.log(`Grew from ${oldSize} to ${oldSize + 10} pages`);
 * ```
 */
interface MemoryAccess {
  /** Read bytes from linear memory at given offset */
  read(offset: number, length: number): Promise<Uint8Array>;
  /** Write bytes to linear memory at given offset */
  write(offset: number, data: Uint8Array): Promise<void>;
  /** Get current memory size in pages (1 page = 64KB) */
  size(): Promise<number>;
  /** Grow memory by specified number of pages, returns old size */
  grow(pages: number): Promise<number>;
}

/**
 * WebAssembly instance.
 *
 * Represents an instantiated WebAssembly module with access to exports,
 * function calls, and linear memory.
 *
 * **Lifecycle:**
 * 1. Compile module: `compile(bytes)` or `compileFile(path)`
 * 2. Instantiate: `instantiate(moduleId, config?)`
 * 3. Use: `call()`, `memory.read()`, etc.
 * 4. Cleanup: `drop()`
 *
 * @example
 * ```typescript
 * import { compile, instantiate } from "runtime:wasm";
 *
 * const moduleId = await compile(wasmBytes);
 * const instance = await instantiate(moduleId);
 *
 * // Call function
 * const [result] = await instance.call("fibonacci", 10);
 * console.log("fib(10) =", result);
 *
 * // List exports
 * const exports = await instance.getExports();
 * console.log("Exports:", exports.map(e => e.name));
 *
 * // Cleanup
 * await instance.drop();
 * ```
 */
interface WasmInstance {
  /** Unique instance identifier */
  readonly id: string;
  /** Module ID this instance was created from */
  readonly moduleId: string;
  /** Call an exported function by name */
  call(name: string, ...args: (number | bigint | WasmValue)[]): Promise<number[]>;
  /** Get list of all exports from this instance */
  getExports(): Promise<WasmExport[]>;
  /** Linear memory access interface */
  memory: MemoryAccess;
  /** Drop this instance and free resources */
  drop(): Promise<void>;
}

const core = Deno.core;

/**
 * Compile WebAssembly bytes into a reusable module.
 *
 * Compiles WebAssembly bytecode and returns a module ID that can be used to create
 * multiple instances. The compiled module is cached in memory and can be instantiated
 * multiple times without recompilation.
 *
 * **Performance:** Compilation is relatively expensive. Cache the module ID and reuse
 * it for multiple instances when possible.
 *
 * @param bytes - WebAssembly bytecode as Uint8Array
 * @returns Module ID (opaque string identifier)
 *
 * @throws Error [5000] if compilation fails (invalid WASM bytecode)
 * @throws Error [5008] if I/O error occurs
 *
 * @example
 * ```typescript
 * import { compile, instantiate } from "runtime:wasm";
 *
 * // Load WASM file
 * const wasmBytes = await Deno.readFile("./module.wasm");
 *
 * // Compile module
 * const moduleId = await compile(wasmBytes);
 * console.log("Compiled module:", moduleId);
 *
 * // Create multiple instances from same module
 * const instance1 = await instantiate(moduleId);
 * const instance2 = await instantiate(moduleId);
 * ```
 *
 * @example
 * ```typescript
 * // Compile from fetched bytes
 * const response = await fetch("https://example.com/module.wasm");
 * const wasmBytes = new Uint8Array(await response.arrayBuffer());
 * const moduleId = await compile(wasmBytes);
 * ```
 */
export async function compile(bytes: Uint8Array): Promise<string> {
  return await core.ops.op_wasm_compile(Array.from(bytes));
}

/**
 * Compile WebAssembly module from a file.
 *
 * Convenience function that loads and compiles a WASM file in one step.
 * Equivalent to reading the file and calling `compile()`, but more efficient.
 *
 * @param path - Path to .wasm file (relative or absolute)
 * @returns Module ID (opaque string identifier)
 *
 * @throws Error [5000] if compilation fails (invalid WASM bytecode)
 * @throws Error [5008] if file not found or cannot be read
 * @throws Error [5009] if permission denied
 *
 * @example
 * ```typescript
 * import { compileFile, instantiate } from "runtime:wasm";
 *
 * // Compile from file
 * const moduleId = await compileFile("./add.wasm");
 *
 * // Instantiate
 * const instance = await instantiate(moduleId);
 * const [result] = await instance.call("add", 10, 32);
 * console.log("10 + 32 =", result); // 42
 * ```
 *
 * @example
 * ```typescript
 * // Compile with error handling
 * try {
 *   const moduleId = await compileFile("./module.wasm");
 *   console.log("Compiled successfully");
 * } catch (error) {
 *   if (error.message.includes("[5000]")) {
 *     console.error("Invalid WASM file");
 *   } else if (error.message.includes("[5008]")) {
 *     console.error("File not found");
 *   }
 * }
 * ```
 */
export async function compileFile(path: string): Promise<string> {
  return await core.ops.op_wasm_compile_file(path);
}

/**
 * Drop a compiled module and free its resources.
 *
 * Removes the compiled module from the cache. All instances created from this
 * module must be dropped before calling this function.
 *
 * **Important:** Drop instances before dropping the module they were created from.
 *
 * @param moduleId - Module ID returned by compile() or compileFile()
 *
 * @throws Error [5004] if module ID is invalid
 *
 * @example
 * ```typescript
 * import { compile, instantiate, dropModule } from "runtime:wasm";
 *
 * const moduleId = await compile(wasmBytes);
 * const instance = await instantiate(moduleId);
 *
 * // Use instance...
 * await instance.call("main");
 *
 * // Cleanup: drop instance first, then module
 * await instance.drop();
 * await dropModule(moduleId);
 * ```
 *
 * @example
 * ```typescript
 * // Multiple instances
 * const moduleId = await compile(wasmBytes);
 * const instances = await Promise.all([
 *   instantiate(moduleId),
 *   instantiate(moduleId),
 *   instantiate(moduleId)
 * ]);
 *
 * // Cleanup all instances
 * await Promise.all(instances.map(inst => inst.drop()));
 *
 * // Now safe to drop module
 * await dropModule(moduleId);
 * ```
 */
export async function dropModule(moduleId: string): Promise<void> {
  return await core.ops.op_wasm_drop_module(moduleId);
}

/**
 * Instantiate a compiled WebAssembly module.
 *
 * Creates a new instance from a compiled module. Multiple instances can be created
 * from the same compiled module. Each instance has its own linear memory and state.
 *
 * Optionally configure WASI (WebAssembly System Interface) to provide file system
 * access, environment variables, command-line arguments, and standard I/O to the
 * WebAssembly module.
 *
 * @param moduleId - Module ID returned by compile() or compileFile()
 * @param wasiConfig - Optional WASI configuration for system interface
 * @returns WasmInstance object with call(), getExports(), memory, and drop() methods
 *
 * @throws Error [5001] if instantiation fails
 * @throws Error [5004] if module ID is invalid
 * @throws Error [5010] if WASI configuration is invalid
 * @throws Error [5009] if permission denied for preopen paths
 *
 * @example
 * ```typescript
 * import { compile, instantiate } from "runtime:wasm";
 *
 * // Basic instantiation
 * const moduleId = await compile(wasmBytes);
 * const instance = await instantiate(moduleId);
 *
 * // Call exported function
 * const [result] = await instance.call("add", 10, 32);
 * console.log("10 + 32 =", result); // 42
 *
 * // Cleanup
 * await instance.drop();
 * ```
 *
 * @example
 * ```typescript
 * // Instantiate with WASI configuration
 * const instance = await instantiate(moduleId, {
 *   // Map guest paths to host directories (capability-based security)
 *   preopens: {
 *     "/data": "./app-data",
 *     "/config": "./config"
 *   },
 *   // Provide environment variables
 *   env: {
 *     "DATABASE_URL": "sqlite:///data/app.db",
 *     "LOG_LEVEL": "debug"
 *   },
 *   // Command-line arguments
 *   args: ["--verbose", "--port", "3000"],
 *   // Inherit standard I/O from host
 *   inheritStdout: true,
 *   inheritStderr: true
 * });
 *
 * // Module can now access /data and /config directories
 * await instance.call("main");
 * ```
 *
 * @example
 * ```typescript
 * // Create multiple instances from same module
 * const moduleId = await compile(wasmBytes);
 *
 * const instances = await Promise.all([
 *   instantiate(moduleId, { env: { "WORKER_ID": "1" } }),
 *   instantiate(moduleId, { env: { "WORKER_ID": "2" } }),
 *   instantiate(moduleId, { env: { "WORKER_ID": "3" } })
 * ]);
 *
 * // Each instance has independent state and memory
 * for (const inst of instances) {
 *   await inst.call("process_data");
 * }
 *
 * // Cleanup all instances
 * await Promise.all(instances.map(inst => inst.drop()));
 * await dropModule(moduleId);
 * ```
 *
 * @example
 * ```typescript
 * // Access instance methods
 * const instance = await instantiate(moduleId);
 *
 * // List available exports
 * const exports = await instance.getExports();
 * console.log("Functions:", exports.filter(e => e.kind === "func").map(e => e.name));
 *
 * // Read/write linear memory
 * const data = await instance.memory.read(0, 256);
 * console.log("Memory at 0:", data);
 *
 * await instance.memory.write(100, new Uint8Array([1, 2, 3, 4]));
 *
 * // Check memory size (in 64KB pages)
 * const pages = await instance.memory.size();
 * console.log(`Memory: ${pages} pages (${pages * 64}KB)`);
 * ```
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
     * Call an exported WebAssembly function.
     *
     * Invokes an exported function by name with the provided arguments. Arguments
     * are automatically converted to appropriate WASM types based on their JavaScript
     * type and value range:
     * - Integer in i32 range (-2^31 to 2^31-1) -> i32
     * - Integer outside i32 range -> i64
     * - Float -> f64
     * - BigInt -> i64
     * - WasmValue object -> uses specified type
     *
     * For explicit type control, use the `types` helper to create typed values.
     *
     * @param name - Name of the exported function to call
     * @param args - Function arguments (numbers, bigints, or typed WasmValue objects)
     * @returns Array of return values (unwrapped to numbers)
     *
     * @throws Error [5003] if function export not found
     * @throws Error [5002] if function call fails
     * @throws Error [5007] if argument types don't match function signature
     * @throws Error [5005] if instance ID is invalid
     *
     * @example
     * ```typescript
     * // Simple function call with automatic type conversion
     * const [result] = await instance.call("add", 10, 32);
     * console.log(result); // 42
     * ```
     *
     * @example
     * ```typescript
     * // Explicit type control using types helper
     * import { types } from "runtime:wasm";
     *
     * const [result] = await instance.call("multiply",
     *   types.i32(7),
     *   types.i32(6)
     * );
     * console.log(result); // 42
     * ```
     *
     * @example
     * ```typescript
     * // Call function with multiple return values
     * const [quotient, remainder] = await instance.call("divmod", 42, 5);
     * console.log(`42 / 5 = ${quotient} remainder ${remainder}`); // 8 remainder 2
     * ```
     *
     * @example
     * ```typescript
     * // Call function with 64-bit integers
     * const [result] = await instance.call("add_i64",
     *   types.i64(9007199254740991n),  // Max safe integer + 1
     *   types.i64(1n)
     * );
     * console.log(result);
     * ```
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
     * Get list of all exports from this WebAssembly instance.
     *
     * Returns metadata about all exported items including functions, memory,
     * tables, and globals. Use this to introspect available functionality
     * before calling functions or accessing exports.
     *
     * @returns Array of export metadata (name, kind, and type information)
     *
     * @throws Error [5005] if instance ID is invalid
     *
     * @example
     * ```typescript
     * const exports = await instance.getExports();
     *
     * // List all exported functions
     * const functions = exports.filter(e => e.kind === "func");
     * console.log("Available functions:", functions.map(f => f.name));
     *
     * // Find memory export
     * const memory = exports.find(e => e.kind === "memory");
     * if (memory) {
     *   console.log("Memory export:", memory.name);
     * }
     * ```
     *
     * @example
     * ```typescript
     * // Check if specific function exists before calling
     * const exports = await instance.getExports();
     * const hasAdd = exports.some(e => e.kind === "func" && e.name === "add");
     *
     * if (hasAdd) {
     *   const [result] = await instance.call("add", 10, 32);
     *   console.log(result);
     * }
     * ```
     */
    async getExports(): Promise<WasmExport[]> {
      return await core.ops.op_wasm_get_exports(instanceId);
    },

    /**
     * Linear memory access interface.
     *
     * Provides direct access to the WebAssembly module's linear memory for reading
     * and writing bytes, checking memory size, and growing memory dynamically.
     *
     * WebAssembly memory is organized in 64KB pages and can be shared between the
     * host (JavaScript) and the guest (WebAssembly module).
     */
    memory: {
      /**
       * Read bytes from WebAssembly linear memory.
       *
       * Reads a contiguous block of memory starting at the specified offset.
       * Memory access is bounds-checked - attempting to read beyond memory
       * size will throw an error.
       *
       * @param offset - Byte offset to start reading from (0-based)
       * @param length - Number of bytes to read
       * @returns Uint8Array containing the read bytes
       *
       * @throws Error [5006] if offset + length exceeds memory size
       * @throws Error [5005] if instance ID is invalid
       *
       * @example
       * ```typescript
       * // Read first 256 bytes of memory
       * const header = await instance.memory.read(0, 256);
       * console.log("Memory header:", header);
       * ```
       *
       * @example
       * ```typescript
       * // Read string from memory (null-terminated)
       * const bytes = await instance.memory.read(1024, 256);
       * const nullIndex = bytes.indexOf(0);
       * const text = new TextDecoder().decode(bytes.slice(0, nullIndex));
       * console.log("String at 1024:", text);
       * ```
       */
      async read(offset: number, length: number): Promise<Uint8Array> {
        const bytes = await core.ops.op_wasm_memory_read(instanceId, offset, length);
        return new Uint8Array(bytes);
      },

      /**
       * Write bytes to WebAssembly linear memory.
       *
       * Writes a contiguous block of bytes to memory starting at the specified
       * offset. Memory access is bounds-checked - attempting to write beyond
       * memory size will throw an error.
       *
       * @param offset - Byte offset to start writing at (0-based)
       * @param data - Bytes to write to memory
       *
       * @throws Error [5006] if offset + data.length exceeds memory size
       * @throws Error [5005] if instance ID is invalid
       *
       * @example
       * ```typescript
       * // Write string to memory
       * const text = "Hello, WASM!";
       * const bytes = new TextEncoder().encode(text);
       * await instance.memory.write(0, bytes);
       * ```
       *
       * @example
       * ```typescript
       * // Write binary data to memory
       * const data = new Uint8Array([0x48, 0x65, 0x6c, 0x6c, 0x6f]);
       * await instance.memory.write(1024, data);
       *
       * // WASM module can now read the data at offset 1024
       * await instance.call("process_data", 1024, data.length);
       * ```
       */
      async write(offset: number, data: Uint8Array): Promise<void> {
        return await core.ops.op_wasm_memory_write(instanceId, offset, Array.from(data));
      },

      /**
       * Get current memory size in pages.
       *
       * Returns the current size of linear memory in 64KB pages. To convert
       * to bytes, multiply by 65536 (64 * 1024).
       *
       * @returns Number of 64KB pages currently allocated
       *
       * @throws Error [5005] if instance ID is invalid
       *
       * @example
       * ```typescript
       * const pages = await instance.memory.size();
       * const bytes = pages * 65536;
       * console.log(`Memory: ${pages} pages (${bytes} bytes, ${bytes / 1024}KB)`);
       * ```
       *
       * @example
       * ```typescript
       * // Check if memory needs to grow before writing
       * const requiredBytes = 1024 * 1024; // 1MB
       * const currentBytes = (await instance.memory.size()) * 65536;
       *
       * if (currentBytes < requiredBytes) {
       *   const pagesNeeded = Math.ceil((requiredBytes - currentBytes) / 65536);
       *   await instance.memory.grow(pagesNeeded);
       * }
       * ```
       */
      async size(): Promise<number> {
        return await core.ops.op_wasm_memory_size(instanceId);
      },

      /**
       * Grow linear memory by the specified number of pages.
       *
       * Attempts to increase memory size by the requested number of 64KB pages.
       * Returns the previous memory size (in pages) before growth. If growth
       * fails (e.g., exceeds maximum), throws an error.
       *
       * **Note:** Growing memory can be expensive. Consider allocating sufficient
       * initial memory in your WASM module to minimize runtime growth operations.
       *
       * @param pages - Number of 64KB pages to add to memory
       * @returns Previous memory size in pages (before growth)
       *
       * @throws Error [5006] if memory cannot be grown (exceeds maximum)
       * @throws Error [5005] if instance ID is invalid
       *
       * @example
       * ```typescript
       * // Grow memory by 1 page (64KB)
       * const oldSize = await instance.memory.grow(1);
       * const newSize = await instance.memory.size();
       * console.log(`Memory grew from ${oldSize} to ${newSize} pages`);
       * ```
       *
       * @example
       * ```typescript
       * // Ensure at least 1MB of memory
       * const currentPages = await instance.memory.size();
       * const requiredPages = Math.ceil(1024 * 1024 / 65536); // 16 pages = 1MB
       *
       * if (currentPages < requiredPages) {
       *   const pagesToAdd = requiredPages - currentPages;
       *   await instance.memory.grow(pagesToAdd);
       *   console.log(`Grew memory by ${pagesToAdd} pages`);
       * }
       * ```
       */
      async grow(pages: number): Promise<number> {
        return await core.ops.op_wasm_memory_grow(instanceId, pages);
      },
    },

    /**
     * Drop this instance and free its resources.
     *
     * Destroys the WebAssembly instance and releases all associated resources
     * including linear memory. After dropping, this instance cannot be used
     * for any operations.
     *
     * **Important:** Always drop instances when done to prevent memory leaks.
     * Drop instances before dropping the module they were created from.
     *
     * @throws Error [5005] if instance ID is invalid
     *
     * @example
     * ```typescript
     * const instance = await instantiate(moduleId);
     *
     * // Use instance...
     * await instance.call("process");
     *
     * // Cleanup when done
     * await instance.drop();
     * ```
     *
     * @example
     * ```typescript
     * // Cleanup multiple instances
     * const instances = await Promise.all([
     *   instantiate(moduleId),
     *   instantiate(moduleId),
     *   instantiate(moduleId)
     * ]);
     *
     * // Process in parallel
     * await Promise.all(instances.map(inst => inst.call("work")));
     *
     * // Cleanup all instances
     * await Promise.all(instances.map(inst => inst.drop()));
     *
     * // Now safe to drop the module
     * await dropModule(moduleId);
     * ```
     */
    async drop(): Promise<void> {
      return await core.ops.op_wasm_drop_instance(instanceId);
    },
  };
}

/**
 * Call a WebAssembly function (low-level API).
 *
 * Low-level function call API that requires explicit instance ID and typed
 * WasmValue arguments. For most use cases, prefer `instance.call()` which
 * provides automatic type conversion and a cleaner API.
 *
 * This function is useful when you need to store instance IDs separately
 * or when working with the raw protocol.
 *
 * @param instanceId - Instance ID from instantiate()
 * @param funcName - Name of exported function to call
 * @param args - Array of typed WasmValue arguments
 * @returns Array of typed WasmValue results
 *
 * @throws Error [5003] if function export not found
 * @throws Error [5002] if function call fails
 * @throws Error [5007] if argument types don't match function signature
 * @throws Error [5005] if instance ID is invalid
 *
 * @example
 * ```typescript
 * import { compile, instantiate, call, types } from "runtime:wasm";
 *
 * const moduleId = await compile(wasmBytes);
 * const instance = await instantiate(moduleId);
 *
 * // Low-level call with explicit types
 * const results = await call(instance.id, "add", [
 *   types.i32(10),
 *   types.i32(32)
 * ]);
 *
 * console.log("Result:", results[0].value); // 42
 * ```
 *
 * @example
 * ```typescript
 * // Prefer instance.call() for cleaner API
 * const [result] = await instance.call("add", 10, 32);
 * console.log("Result:", result); // 42 (unwrapped)
 * ```
 */
export async function call(instanceId: string, funcName: string, args: WasmValue[]): Promise<WasmValue[]> {
  return await core.ops.op_wasm_call(instanceId, funcName, args);
}

/**
 * Get exports from a WebAssembly instance (low-level API).
 *
 * Low-level exports query API that requires explicit instance ID.
 * For most use cases, prefer `instance.getExports()` which provides
 * a cleaner API.
 *
 * @param instanceId - Instance ID from instantiate()
 * @returns Array of export metadata
 *
 * @throws Error [5005] if instance ID is invalid
 *
 * @example
 * ```typescript
 * import { compile, instantiate, getExports } from "runtime:wasm";
 *
 * const moduleId = await compile(wasmBytes);
 * const instance = await instantiate(moduleId);
 *
 * // Low-level exports query
 * const exports = await getExports(instance.id);
 * console.log("Exports:", exports);
 * ```
 *
 * @example
 * ```typescript
 * // Prefer instance.getExports() for cleaner API
 * const exports = await instance.getExports();
 * console.log("Exports:", exports);
 * ```
 */
export async function getExports(instanceId: string): Promise<WasmExport[]> {
  return await core.ops.op_wasm_get_exports(instanceId);
}

/**
 * Helper object for creating typed WebAssembly values.
 *
 * Provides convenience functions to create explicitly typed WasmValue objects.
 * Use these when you need precise control over WASM value types, especially
 * when automatic type conversion doesn't match your needs.
 *
 * @example
 * ```typescript
 * import { types } from "runtime:wasm";
 *
 * // Create typed values
 * const a = types.i32(42);        // 32-bit integer
 * const b = types.i64(1000000n);  // 64-bit integer (BigInt)
 * const c = types.f32(3.14);      // 32-bit float
 * const d = types.f64(2.71828);   // 64-bit float
 *
 * // Pass to function call
 * await instance.call("compute", a, b, c, d);
 * ```
 *
 * @example
 * ```typescript
 * // Ensure specific type even for small integers
 * const value = types.i64(42);  // Force i64 instead of auto i32
 * await instance.call("process_i64", value);
 * ```
 */
export const types = {
  /**
   * Create a 32-bit integer value.
   *
   * Range: -2,147,483,648 to 2,147,483,647 (-2^31 to 2^31-1)
   *
   * @param value - JavaScript number (will be truncated to i32 range)
   * @returns WasmValue with type 'i32'
   *
   * @example
   * ```typescript
   * const val = types.i32(42);
   * await instance.call("add_i32", val, types.i32(8));
   * ```
   */
  i32: (value: number): WasmValue => ({ type: 'i32', value }),

  /**
   * Create a 64-bit integer value.
   *
   * Range: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 (-2^63 to 2^63-1)
   *
   * @param value - JavaScript number or BigInt
   * @returns WasmValue with type 'i64'
   *
   * @example
   * ```typescript
   * const val = types.i64(9007199254740991n);  // Use BigInt for large values
   * await instance.call("process_i64", val);
   * ```
   */
  i64: (value: number | bigint): WasmValue => ({ type: 'i64', value: typeof value === 'bigint' ? Number(value) : value }),

  /**
   * Create a 32-bit floating point value (single precision).
   *
   * Precision: ~7 decimal digits
   *
   * @param value - JavaScript number
   * @returns WasmValue with type 'f32'
   *
   * @example
   * ```typescript
   * const pi = types.f32(3.14159);
   * await instance.call("compute_circle", pi);
   * ```
   */
  f32: (value: number): WasmValue => ({ type: 'f32', value }),

  /**
   * Create a 64-bit floating point value (double precision).
   *
   * Precision: ~15-17 decimal digits
   *
   * @param value - JavaScript number
   * @returns WasmValue with type 'f64'
   *
   * @example
   * ```typescript
   * const e = types.f64(2.718281828459045);
   * await instance.call("compute_exp", e);
   * ```
   */
  f64: (value: number): WasmValue => ({ type: 'f64', value }),
};

/**
 * Low-level memory operations namespace.
 *
 * Provides direct access to WebAssembly linear memory operations using
 * instance IDs. For most use cases, prefer `instance.memory` which provides
 * a cleaner API without requiring explicit instance IDs.
 *
 * @example
 * ```typescript
 * import { memory } from "runtime:wasm";
 *
 * // Low-level memory operations
 * const data = await memory.read(instanceId, 0, 256);
 * await memory.write(instanceId, 1024, new Uint8Array([1, 2, 3]));
 * const pages = await memory.size(instanceId);
 * const oldSize = await memory.grow(instanceId, 1);
 * ```
 *
 * @example
 * ```typescript
 * // Prefer instance.memory for cleaner API
 * const data = await instance.memory.read(0, 256);
 * await instance.memory.write(1024, new Uint8Array([1, 2, 3]));
 * ```
 */
export const memory = {
  /**
   * Read bytes from WebAssembly linear memory (low-level API).
   *
   * @param instanceId - Instance ID from instantiate()
   * @param offset - Byte offset to start reading from
   * @param length - Number of bytes to read
   * @returns Uint8Array containing the read bytes
   *
   * @throws Error [5006] if offset + length exceeds memory size
   * @throws Error [5005] if instance ID is invalid
   *
   * @example
   * ```typescript
   * const bytes = await memory.read(instanceId, 0, 100);
   * console.log("Memory:", bytes);
   * ```
   */
  async read(instanceId: string, offset: number, length: number): Promise<Uint8Array> {
    const bytes = await core.ops.op_wasm_memory_read(instanceId, offset, length);
    return new Uint8Array(bytes);
  },

  /**
   * Write bytes to WebAssembly linear memory (low-level API).
   *
   * @param instanceId - Instance ID from instantiate()
   * @param offset - Byte offset to start writing at
   * @param data - Bytes to write to memory
   *
   * @throws Error [5006] if offset + data.length exceeds memory size
   * @throws Error [5005] if instance ID is invalid
   *
   * @example
   * ```typescript
   * await memory.write(instanceId, 1024, new Uint8Array([1, 2, 3, 4]));
   * ```
   */
  async write(instanceId: string, offset: number, data: Uint8Array): Promise<void> {
    return await core.ops.op_wasm_memory_write(instanceId, offset, Array.from(data));
  },

  /**
   * Get current memory size in pages (low-level API).
   *
   * @param instanceId - Instance ID from instantiate()
   * @returns Number of 64KB pages currently allocated
   *
   * @throws Error [5005] if instance ID is invalid
   *
   * @example
   * ```typescript
   * const pages = await memory.size(instanceId);
   * console.log(`Memory: ${pages} pages (${pages * 64}KB)`);
   * ```
   */
  async size(instanceId: string): Promise<number> {
    return await core.ops.op_wasm_memory_size(instanceId);
  },

  /**
   * Grow linear memory by the specified number of pages (low-level API).
   *
   * @param instanceId - Instance ID from instantiate()
   * @param pages - Number of 64KB pages to add to memory
   * @returns Previous memory size in pages (before growth)
   *
   * @throws Error [5006] if memory cannot be grown (exceeds maximum)
   * @throws Error [5005] if instance ID is invalid
   *
   * @example
   * ```typescript
   * const oldSize = await memory.grow(instanceId, 1);
   * console.log(`Memory grew from ${oldSize} pages`);
   * ```
   */
  async grow(instanceId: string, pages: number): Promise<number> {
    return await core.ops.op_wasm_memory_grow(instanceId, pages);
  },
};
