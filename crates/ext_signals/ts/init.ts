// runtime:signals module - subscribe to OS signals (Unix only).

declare const Deno: {
  core: {
    ops: {
      op_signals_supported(): string[];
      op_signals_subscribe(signals: string[]): Promise<bigint>;
      op_signals_next(id: bigint): Promise<SignalEvent | null>;
      op_signals_unsubscribe(id: bigint): boolean;
    };
  };
};

interface SignalEvent {
  signal: string;
}

interface SignalSubscription {
  id: bigint;
  next(): Promise<SignalEvent | null>;
  unsubscribe(): Promise<boolean>;
}

const core = Deno.core;

export function supportedSignals(): string[] {
  return core.ops.op_signals_supported();
}

export async function subscribe(signals: string[]): Promise<SignalSubscription> {
  const id = await core.ops.op_signals_subscribe(signals);

  return {
    id,
    async next(): Promise<SignalEvent | null> {
      return await core.ops.op_signals_next(id);
    },
    async unsubscribe(): Promise<boolean> {
      return core.ops.op_signals_unsubscribe(id);
    },
  };
}
