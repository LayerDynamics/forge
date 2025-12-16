// runtime:log module - structured logging bridge to host tracing.

interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

type LogLevel = "trace" | "debug" | "info" | "warn" | "warning" | "error";

declare const Deno: {
  core: {
    ops: {
      op_log_info(): ExtensionInfo;
      op_log_emit(level: string, message: string, fields?: Record<string, unknown>): void;
    };
  };
};

const { core } = Deno;

export function info(): ExtensionInfo {
  return core.ops.op_log_info();
}

export function emit(level: LogLevel, message: string, fields?: Record<string, unknown>): void {
  core.ops.op_log_emit(level, message, fields ?? {});
}

export function trace(message: string, fields?: Record<string, unknown>): void {
  emit("trace", message, fields);
}

export function debug(message: string, fields?: Record<string, unknown>): void {
  emit("debug", message, fields);
}

export function infoLog(message: string, fields?: Record<string, unknown>): void {
  emit("info", message, fields);
}

export function warn(message: string, fields?: Record<string, unknown>): void {
  emit("warn", message, fields);
}

export function error(message: string, fields?: Record<string, unknown>): void {
  emit("error", message, fields);
}
