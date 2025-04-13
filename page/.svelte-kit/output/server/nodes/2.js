

export const index = 2;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_page.svelte.js')).default;
export const imports = ["_app/immutable/nodes/2.4lZMz2L_.js","_app/immutable/chunks/BRAdFc7x.js","_app/immutable/chunks/YzQQLn1s.js","_app/immutable/chunks/1D_4X9-L.js"];
export const stylesheets = [];
export const fonts = [];
