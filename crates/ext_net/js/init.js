// host:net module - JavaScript wrapper for Deno core ops
const core = Deno.core;

export async function fetch(url, opts = {}) {
  const response = await core.ops.op_net_fetch(url, opts);
  return {
    ...response,
    statusText: response.status_text || response.statusText || "",
  };
}

export async function fetchBytes(url, opts = {}) {
  const response = await core.ops.op_net_fetch_bytes(url, opts);
  return {
    ...response,
    statusText: response.status_text || response.statusText || "",
    body: new Uint8Array(response.body),
  };
}

export async function fetchJson(url, opts = {}) {
  const response = await fetch(url, {
    ...opts,
    headers: {
      "Accept": "application/json",
      ...opts.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`HTTP error ${response.status}: ${response.statusText}`);
  }

  return JSON.parse(response.body);
}

export async function postJson(url, data, opts = {}) {
  return fetch(url, {
    ...opts,
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...opts.headers,
    },
    body: JSON.stringify(data),
  });
}
