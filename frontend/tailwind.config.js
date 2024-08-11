/** @type {import('tailwindcss').Config} */
module.exports = {
	darkMode: ["class"],
	content: [
		"./pages/**/*.{js,jsx}",
		"./components/**/*.{js,jsx}",
		"./app/**/*.{js,jsx}",
		"./src/**/*.{js,jsx}",
	],
	prefix: "",
	theme: {
		container: {
			center: true,
			padding: "2rem",
			screens: {
				"2xl": "1400px",
			},
		},
		extend: {
			colors: {
				border: "hsl(var(--border))",
				input: "hsl(var(--input))",
				ring: "hsl(var(--ring))",
				background: "#000",
				foreground: "#fff",
				primary: {
					DEFAULT: "#ffffff",
					foreground: "#000000"
				},
				secondary: {
					DEFAULT: "#777",
					foreground: "#fff",
				},
				destructive: {
					DEFAULT: "rgb(153 27 27)",
					foreground: "#000",
				},
				muted: {
					DEFAULT: "#333",
					foreground: "#555",
				},
				accent: {
					DEFAULT: "rgb(22 78 99)",
					foreground: "#fff",
				},
				popover: {
					DEFAULT: "#222",
					foreground: "#fff",
				},
				card: {
					DEFAULT: "#222",
					foreground: "#fff",
				},
			},
			keyframes: {
				"expand-width": {
					"0%": { width: "0px" },
				},
				"accordion-down": {
					from: { height: "0" },
					to: { height: "var(--radix-accordion-content-height)" },
				},
				"accordion-up": {
					from: { height: "var(--radix-accordion-content-height)" },
					to: { height: "0" },
				},
				"fade-in": {
                    "0%": {
                        opacity: 0
                    },
                    "100%": {
                        opacity: 1
                    },
                },
			},
			animation: {
				"expand-width": "expand-width 0.75s ease-in-out",
				"accordion-down": "accordion-down 0.2s ease-out",
				"accordion-up": "accordion-up 0.2s ease-out",
				"fadein": 'fade-in ease-in-out',
			},
			borderRadius: {
				lg: "var(--radius)",
				md: "calc(var(--radius) - 2px)",
				sm: "calc(var(--radius) - 4px)",
			},
		},
	},
	plugins: [require("tailwindcss-animate")],
};
