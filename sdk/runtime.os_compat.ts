// runtime:os_compat bindings
export interface OsInfo {
  os: string;
  arch: string;
  family: string;
  path_sep: string;
  env_sep: string;
  tmp_dir: string;
  home_dir: string | null;
}

declare const Deno: {
  core: {
    ops: {
      op_os_compat_info(): OsInfo;
      op_os_compat_path_sep(): string;
    };
  };
};

const { core } = Deno;
const ops = {
  info: core.ops.op_os_compat_info,
  pathSep: core.ops.op_os_compat_path_sep,
};

export function info(): OsInfo {
  return ops.info();
}

export function pathSep(): string {
  return ops.pathSep();
}