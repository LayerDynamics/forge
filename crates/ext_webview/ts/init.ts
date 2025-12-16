// runtime:webview extension
// Provides a small wrapper around the runtime:window ops to manage webviews.

declare const Deno: {
  core: {
    ops: {
      op_host_webview_new(opts: WebViewOptions): WebViewHandle;
      op_host_webview_exit(params: { id: string }): void;
      op_host_webview_eval(params: { id: string; js: string }): void;
      op_host_webview_set_color(params: { id: string; r: number; g: number; b: number; a: number }): void;
      op_host_webview_set_title(params: { id: string; title: string }): void;
      op_host_webview_set_fullscreen(params: { id: string; fullscreen: boolean }): void;
      op_host_webview_loop(params: { id: string; blocking: number }): Promise<{ code: number }>;
      op_host_webview_run(params: { id: string }): Promise<void>;
    };
  };
};

// deno-lint-ignore no-explicit-any
const core = (Deno as any).core;

export interface WebViewOptions {
  title: string;
  url: string;
  width: number;
  height: number;
  resizable: boolean;
  debug: boolean;
  frameless: boolean;
}

export interface WebViewHandle {
  id: string;
}

function webviewNew(opts: WebViewOptions): WebViewHandle {
  return core.ops.op_host_webview_new(opts);
}

function webviewExit(id: string): void {
  core.ops.op_host_webview_exit({ id });
}

function webviewEval(id: string, js: string): void {
  core.ops.op_host_webview_eval({ id, js });
}

function webviewSetColor(id: string, r: number, g: number, b: number, a: number): void {
  core.ops.op_host_webview_set_color({ id, r, g, b, a });
}

function webviewSetTitle(id: string, title: string): void {
  core.ops.op_host_webview_set_title({ id, title });
}

function webviewSetFullscreen(id: string, fullscreen: boolean): void {
  core.ops.op_host_webview_set_fullscreen({ id, fullscreen });
}

async function webviewLoop(id: string, blocking: number): Promise<{ code: number }> {
  return await core.ops.op_host_webview_loop({ id, blocking });
}

async function webviewRun(id: string): Promise<void> {
  await core.ops.op_host_webview_run({ id });
}

// Aliases with friendlier names
const newWebView = webviewNew;
const exitWebView = webviewExit;
const evalInWebView = webviewEval;
const setWebViewColor = webviewSetColor;
const setWebViewTitle = webviewSetTitle;
const setWebViewFullscreen = webviewSetFullscreen;
const webViewLoop = webviewLoop;
const runWebView = webviewRun;

export {
  // primary names
  webviewNew,
  webviewExit,
  webviewEval,
  webviewSetColor,
  webviewSetTitle,
  webviewSetFullscreen,
  webviewLoop,
  webviewRun,
  // aliases
  newWebView,
  exitWebView,
  evalInWebView,
  setWebViewColor,
  setWebViewTitle,
  setWebViewFullscreen,
  webViewLoop,
  runWebView,
};
