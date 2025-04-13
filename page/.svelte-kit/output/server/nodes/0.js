

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const imports = ["_app/immutable/nodes/0.iFjDm3kC.js","_app/immutable/chunks/BRAdFc7x.js","_app/immutable/chunks/YzQQLn1s.js"];
export const stylesheets = ["_app/immutable/assets/0.BYBPKacq.css"];
export const fonts = [];
