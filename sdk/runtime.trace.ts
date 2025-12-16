// runtime:trace extension bindings for spans and instant events.

export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

export interface SpanRecord {
  id: bigint;
  name: string;
  started_at: bigint;
  duration_ms: number;
  attributes?: unknown;
  result?: unknown;
}

declare const Deno: {
  core: {
    ops: {
      op_trace_info(): ExtensionInfo;
      op_trace_start(name: string, attributes?: unknown): bigint;
      op_trace_end(id: bigint, result?: unknown): SpanRecord;
      op_trace_instant(name: string, attributes?: unknown): SpanRecord;
      op_trace_flush(): SpanRecord[];
    };
  };
};

const { core } = Deno;

export function info(): ExtensionInfo {
  return core.ops.op_trace_info();
}

export function start(name: string, attributes?: unknown): bigint {
  return core.ops.op_trace_start(name, attributes);
}

export function end(id: bigint, result?: unknown): SpanRecord {
  return core.ops.op_trace_end(id, result);
}

export function instant(name: string, attributes?: unknown): SpanRecord {
  return core.ops.op_trace_instant(name, attributes);
}

export function flush(): SpanRecord[] {
  return core.ops.op_trace_flush();
}