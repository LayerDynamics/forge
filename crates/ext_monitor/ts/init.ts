// runtime:monitor extension bindings

interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

declare const Deno: {
  core: {
    ops: {
      op_monitor_info(): ExtensionInfo;
      op_monitor_echo(message: string): string;
    };
  };
};

const { core } = Deno;
const ops = {
  info: core.ops.op_monitor_info,
  echo: core.ops.op_monitor_echo,
};

export function info(): ExtensionInfo {
  return ops.info();
}

export function echo(message: string): string {
  return ops.echo(message);
}
