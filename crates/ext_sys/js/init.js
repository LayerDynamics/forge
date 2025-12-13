// host:sys module - JavaScript wrapper for Deno core ops
const core = Deno.core;

export function info() {
  const result = core.ops.op_sys_info();
  return {
    ...result,
    cpuCount: result.cpu_count,
  };
}

export function getEnv(key) {
  return core.ops.op_sys_env_get(key);
}

export function setEnv(key, value) {
  core.ops.op_sys_env_set(key, value);
}

export function cwd() {
  return core.ops.op_sys_cwd();
}

export function homeDir() {
  return core.ops.op_sys_home_dir();
}

export function tempDir() {
  return core.ops.op_sys_temp_dir();
}

export const clipboard = {
  async read() {
    return await core.ops.op_sys_clipboard_read();
  },

  async write(text) {
    return await core.ops.op_sys_clipboard_write(text);
  },
};

export async function notify(title, body = "") {
  return await core.ops.op_sys_notify(title, body);
}

export async function notifyExt(opts) {
  return await core.ops.op_sys_notify_ext(opts);
}

export async function powerInfo() {
  return await core.ops.op_sys_power_info();
}
