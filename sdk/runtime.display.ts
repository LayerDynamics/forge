// runtime:display extension bindings

export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

declare const Deno: {
  core: {
    ops: {
      op_display_info(): ExtensionInfo;
      op_display_echo(message: string): string;
    };
  };
};

const { core } = Deno;
const ops = {
  info: core.ops.op_display_info,
  echo: core.ops.op_display_echo,
};

export function info(): ExtensionInfo {
  return ops.info();
}

export function echo(message: string): string {
  return ops.echo(message);
}