

export const index = 2;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_page.svelte.js')).default;
export const imports = ["_app/immutable/nodes/2.BQ8fOPKe.js","_app/immutable/chunks/DgFyK5i7.js","_app/immutable/chunks/Y270MJa4.js"];
export const stylesheets = ["_app/immutable/assets/2.CevYfuGa.css"];
export const fonts = [];
