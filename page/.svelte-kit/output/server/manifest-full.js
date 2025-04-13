export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set(["favicon.png"]),
	mimeTypes: {".png":"image/png"},
	_: {
		client: {start:"_app/immutable/entry/start.DHLSUz0-.js",app:"_app/immutable/entry/app.DGF5Xj59.js",imports:["_app/immutable/entry/start.DHLSUz0-.js","_app/immutable/chunks/B0LAEEDJ.js","_app/immutable/chunks/YzQQLn1s.js","_app/immutable/chunks/DONkwREh.js","_app/immutable/entry/app.DGF5Xj59.js","_app/immutable/chunks/YzQQLn1s.js","_app/immutable/chunks/B0Z1OCBH.js","_app/immutable/chunks/BRAdFc7x.js","_app/immutable/chunks/DONkwREh.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js'))
		],
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
