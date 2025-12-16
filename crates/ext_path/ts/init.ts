// runtime:path bindings
interface PathParts {
  dir: string;
  base: string;
  ext: string;
}

declare const Deno: {
  core: {
    ops: {
      op_path_join(base: string, segments: string[]): string;
      op_path_dirname(path: string): string;
      op_path_basename(path: string): string;
      op_path_extname(path: string): string;
      op_path_parts(path: string): PathParts;
    };
  };
};

const { core } = Deno;
const ops = {
  join: core.ops.op_path_join,
  dirname: core.ops.op_path_dirname,
  basename: core.ops.op_path_basename,
  extname: core.ops.op_path_extname,
  parts: core.ops.op_path_parts,
};

export function join(base: string, ...segments: string[]): string {
  return ops.join(base, segments);
}

export function dirname(path: string): string {
  return ops.dirname(path);
}

export function basename(path: string): string {
  return ops.basename(path);
}

export function extname(path: string): string {
  return ops.extname(path);
}

export function parts(path: string): PathParts {
  return ops.parts(path);
}
