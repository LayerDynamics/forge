// runtime:lock bindings
interface LockInfo {
  name: string;
  locked: boolean;
}

declare const Deno: {
  core: {
    ops: {
      op_lock_acquire(name: string, timeoutMs: number | null): Promise<bigint>;
      op_lock_try(name: string): Promise<bigint | null>;
      op_lock_release(name: string, token: bigint): boolean;
      op_lock_list(): LockInfo[];
    };
  };
};

const { core } = Deno;
const ops = {
  acquire: core.ops.op_lock_acquire,
  try: core.ops.op_lock_try,
  release: core.ops.op_lock_release,
  list: core.ops.op_lock_list,
};

export async function acquire(name: string, timeoutMs?: number): Promise<bigint> {
  const token = await ops.acquire(name, timeoutMs ?? null);
  return BigInt(token);
}

export async function tryAcquire(name: string): Promise<bigint | null> {
  const token = await ops.try(name);
  return token === null ? null : BigInt(token);
}

export function release(name: string, token: bigint): boolean {
  return ops.release(name, token);
}

export function list(): LockInfo[] {
  return ops.list();
}
