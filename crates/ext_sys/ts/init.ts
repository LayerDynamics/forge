// host:sys module - TypeScript wrapper for Deno core ops

declare const Deno: {
  core: {
    ops: {
      op_sys_info(): RawSystemInfo;
      op_sys_env_get(key: string): string | null;
      op_sys_env_set(key: string, value: string): void;
      op_sys_cwd(): string;
      op_sys_home_dir(): string | null;
      op_sys_temp_dir(): string;
      op_sys_clipboard_read(): Promise<string>;
      op_sys_clipboard_write(text: string): Promise<void>;
      op_sys_notify(title: string, body: string): Promise<void>;
      op_sys_notify_ext(opts: NotifyOptions): Promise<void>;
      op_sys_power_info(): Promise<PowerInfo>;
    };
  };
};

interface RawSystemInfo {
  os: string;
  arch: string;
  hostname: string;
  cpu_count: number;
  cpuCount?: number;
}

interface SystemInfo {
  os: string;
  arch: string;
  hostname: string;
  cpuCount: number;
}

interface NotifyOptions {
  title: string;
  body?: string;
  icon?: string;
  sound?: boolean;
}

interface PowerInfo {
  state: string;
  percentage: number | null;
  timeToFull: number | null;
  timeToEmpty: number | null;
}

const core = Deno.core;

export function info(): SystemInfo {
  const result = core.ops.op_sys_info();
  return {
    os: result.os,
    arch: result.arch,
    hostname: result.hostname,
    cpuCount: result.cpu_count,
  };
}

export function getEnv(key: string): string | null {
  return core.ops.op_sys_env_get(key);
}

export function setEnv(key: string, value: string): void {
  core.ops.op_sys_env_set(key, value);
}

export function cwd(): string {
  return core.ops.op_sys_cwd();
}

export function homeDir(): string | null {
  return core.ops.op_sys_home_dir();
}

export function tempDir(): string {
  return core.ops.op_sys_temp_dir();
}

export const clipboard = {
  async read(): Promise<string> {
    return await core.ops.op_sys_clipboard_read();
  },

  async write(text: string): Promise<void> {
    return await core.ops.op_sys_clipboard_write(text);
  },
};

export async function notify(title: string, body: string = ""): Promise<void> {
  return await core.ops.op_sys_notify(title, body);
}

export async function notifyExt(opts: NotifyOptions): Promise<void> {
  return await core.ops.op_sys_notify_ext(opts);
}

export async function powerInfo(): Promise<PowerInfo> {
  return await core.ops.op_sys_power_info();
}
