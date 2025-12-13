import { openWindow, windowEvents } from "host:ui";
import { compile, instantiate, types } from "host:wasm";
import { readBytes } from "host:fs";

console.log("WASM Forge Example booting...");

// Minimal WASM module with verified bytes
// This is the smallest module that exports a simple add function
// Compiled from: (module (func (export "add") (param i32 i32) (result i32) local.get 0 local.get 1 i32.add))
const ADD_WASM = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, // magic: \0asm
  0x01, 0x00, 0x00, 0x00, // version: 1

  0x01, 0x07,             // type section, 7 bytes
  0x01,                   // 1 type
  0x60, 0x02, 0x7f, 0x7f, // func (param i32 i32)
  0x01, 0x7f,             // (result i32)

  0x03, 0x02,             // function section, 2 bytes
  0x01,                   // 1 function
  0x00,                   // uses type 0

  0x07, 0x07,             // export section, 7 bytes
  0x01,                   // 1 export
  0x03, 0x61, 0x64, 0x64, // name: "add"
  0x00, 0x00,             // func index 0

  0x0a, 0x09,             // code section, 9 bytes
  0x01,                   // 1 function body
  0x07,                   // body size: 7 bytes
  0x00,                   // 0 locals
  0x20, 0x00,             // local.get 0
  0x20, 0x01,             // local.get 1
  0x6a,                   // i32.add
  0x0b,                   // end
]);

// WASM module with multiply function
const MUL_WASM = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, // magic
  0x01, 0x00, 0x00, 0x00, // version

  0x01, 0x07,             // type section
  0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f,

  0x03, 0x02,             // function section
  0x01, 0x00,

  0x07, 0x0c,             // export section, 12 bytes
  0x01,                   // 1 export
  0x08,                   // name length: 8
  0x6d, 0x75, 0x6c, 0x74, 0x69, 0x70, 0x6c, 0x79, // "multiply"
  0x00, 0x00,             // func index 0

  0x0a, 0x09,             // code section
  0x01, 0x07, 0x00,
  0x20, 0x00,             // local.get 0
  0x20, 0x01,             // local.get 1
  0x6c,                   // i32.mul
  0x0b,                   // end
]);

// WASM module with memory operations
// Exports: get_value(ptr: i32) -> i32, set_value(ptr: i32, val: i32), memory
const MEM_WASM = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, // magic
  0x01, 0x00, 0x00, 0x00, // version

  // Type section: 2 types (11 bytes content)
  0x01, 0x0b,             // type section id=1, size=11
  0x02,                   // 2 types
  0x60, 0x01, 0x7f, 0x01, 0x7f, // type 0: (i32) -> i32
  0x60, 0x02, 0x7f, 0x7f, 0x00, // type 1: (i32, i32) -> ()

  // Function section (3 bytes content)
  0x03, 0x03,             // function section id=3, size=3
  0x02,                   // 2 functions
  0x00,                   // get_value uses type 0
  0x01,                   // set_value uses type 1

  // Memory section (3 bytes content)
  0x05, 0x03,             // memory section id=5, size=3
  0x01,                   // 1 memory
  0x00, 0x01,             // min 1 page, no max

  // Export section (34 bytes content)
  // Content: 1(count) + 12(get_value) + 12(set_value) + 9(memory) = 34
  0x07, 0x22,             // export section id=7, size=34 (0x22)
  0x03,                   // 3 exports
  // export "get_value" -> func 0
  0x09,                   // name length: 9
  0x67, 0x65, 0x74, 0x5f, 0x76, 0x61, 0x6c, 0x75, 0x65, // "get_value"
  0x00,                   // export kind: function
  0x00,                   // function index 0
  // export "set_value" -> func 1
  0x09,                   // name length: 9
  0x73, 0x65, 0x74, 0x5f, 0x76, 0x61, 0x6c, 0x75, 0x65, // "set_value"
  0x00,                   // export kind: function
  0x01,                   // function index 1
  // export "memory" -> memory 0
  0x06,                   // name length: 6
  0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, // "memory"
  0x02,                   // export kind: memory
  0x00,                   // memory index 0

  // Code section
  // Content: 1(count) + 8(func0) + 10(func1) = 19 bytes
  0x0a, 0x13,             // code section id=10, size=19 (0x13)
  0x02,                   // 2 functions

  // Function 0: get_value(ptr) -> i32.load(ptr)
  // Body: 0(locals) + local.get(2) + i32.load(3) + end(1) = 7 bytes
  0x07,                   // function body size: 7
  0x00,                   // 0 local declarations
  0x20, 0x00,             // local.get 0
  0x28, 0x02, 0x00,       // i32.load align=2^2=4 offset=0
  0x0b,                   // end

  // Function 1: set_value(ptr, val) -> void
  // Body: 0(locals) + local.get(2) + local.get(2) + i32.store(3) + end(1) = 9 bytes
  0x09,                   // function body size: 9
  0x00,                   // 0 local declarations
  0x20, 0x00,             // local.get 0
  0x20, 0x01,             // local.get 1
  0x36, 0x02, 0x00,       // i32.store align=2^2=4 offset=0
  0x0b,                   // end
]);

interface WasmResult {
  operation: string;
  result: string;
  success: boolean;
}

async function runWasmDemo(): Promise<WasmResult[]> {
  const results: WasmResult[] = [];

  try {
    // === Test 1: Simple add module ===
    console.log("Compiling add WASM module...");
    const addModuleId = await compile(ADD_WASM);
    results.push({
      operation: "Compile add.wasm",
      result: `Module ID: ${addModuleId.slice(0, 8)}...`,
      success: true,
    });

    console.log("Instantiating add module...");
    const addInstance = await instantiate(addModuleId);
    results.push({
      operation: "Instantiate add",
      result: `Instance ID: ${addInstance.id.slice(0, 8)}...`,
      success: true,
    });

    // Get exports from add module
    const addExports = await addInstance.getExports();
    results.push({
      operation: "Get add exports",
      result: addExports.map((e) => `${e.name} (${e.kind})`).join(", "),
      success: addExports.length > 0,
    });

    // Call add function
    const addResult = await addInstance.call("add", 7, 5);
    results.push({
      operation: "add(7, 5)",
      result: `${addResult[0]}`,
      success: addResult[0] === 12,
    });

    // Clean up add instance
    await addInstance.drop();

    // === Test 2: Multiply module ===
    console.log("Compiling multiply WASM module...");
    const mulModuleId = await compile(MUL_WASM);
    results.push({
      operation: "Compile multiply.wasm",
      result: `Module ID: ${mulModuleId.slice(0, 8)}...`,
      success: true,
    });

    const mulInstance = await instantiate(mulModuleId);
    const mulResult = await mulInstance.call("multiply", 6, 7);
    results.push({
      operation: "multiply(6, 7)",
      result: `${mulResult[0]}`,
      success: mulResult[0] === 42,
    });

    await mulInstance.drop();

    // === Test 3: Memory module ===
    console.log("Compiling memory WASM module...");
    const memModuleId = await compile(MEM_WASM);
    results.push({
      operation: "Compile memory.wasm",
      result: `Module ID: ${memModuleId.slice(0, 8)}...`,
      success: true,
    });

    const memInstance = await instantiate(memModuleId);

    // Get exports from memory module
    const memExports = await memInstance.getExports();
    results.push({
      operation: "Get memory exports",
      result: memExports.map((e) => `${e.name} (${e.kind})`).join(", "),
      success: memExports.length === 3,
    });

    // Write to memory using the host API
    const testValue = 12345;
    const valueBytes = new Uint8Array(4);
    new DataView(valueBytes.buffer).setInt32(0, testValue, true);
    await memInstance.memory.write(0, valueBytes);
    results.push({
      operation: `memory.write(0, ${testValue})`,
      result: "OK",
      success: true,
    });

    // Read from memory using WASM function
    const getValue = await memInstance.call("get_value", 0);
    results.push({
      operation: "get_value(0)",
      result: `${getValue[0]}`,
      success: getValue[0] === testValue,
    });

    // Use set_value to write
    await memInstance.call("set_value", 4, 99999);
    results.push({
      operation: "set_value(4, 99999)",
      result: "OK",
      success: true,
    });

    // Read back using memory API
    const memBytes = await memInstance.memory.read(4, 4);
    const memValue = new DataView(memBytes.buffer).getInt32(0, true);
    results.push({
      operation: "memory.read(4, 4)",
      result: `${memValue}`,
      success: memValue === 99999,
    });

    // Get memory size
    const memSize = await memInstance.memory.size();
    results.push({
      operation: "memory.size()",
      result: `${memSize} pages (${memSize * 64}KB)`,
      success: memSize >= 1,
    });

    // Clean up
    await memInstance.drop();
    results.push({
      operation: "Cleanup complete",
      result: "All instances dropped",
      success: true,
    });

  } catch (error) {
    console.error("WASM Demo error:", error);
    results.push({
      operation: "Error",
      result: String(error),
      success: false,
    });
  }

  return results;
}

// Open the window
const win = await openWindow({
  url: "app://index.html",
  width: 800,
  height: 600,
  title: "WASM Forge Example",
});

console.log("Window opened, waiting for ready signal...");

// Handle window events
for await (const event of win.events()) {
  console.log(`Received event: ${event.channel}`, event.payload);

  if (event.channel === "ready") {
    // Run WASM demo and send results to window
    console.log("Running WASM demo...");
    const results = await runWasmDemo();
    await win.send("wasm-results", results);
  }

  if (event.channel === "run-demo") {
    // Re-run demo on request
    console.log("Re-running WASM demo...");
    const results = await runWasmDemo();
    await win.send("wasm-results", results);
  }
}
