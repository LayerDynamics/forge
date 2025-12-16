// runtime:devtools module - open/close devtools for a given window

declare const Deno: {
  core: {
    ops: {
      op_devtools_open(windowId: string): Promise<boolean>;
      op_devtools_close(windowId: string): Promise<boolean>;
      op_devtools_is_open(windowId: string): Promise<boolean>;
    };
  };
};

// deno-lint-ignore no-explicit-any
const core = (Deno as any).core;

export async function open(windowId: string): Promise<boolean> {
  return await core.ops.op_devtools_open(windowId);
}

export async function close(windowId: string): Promise<boolean> {
  return await core.ops.op_devtools_close(windowId);
}

export async function isOpen(windowId: string): Promise<boolean> {
  return await core.ops.op_devtools_is_open(windowId);
}
