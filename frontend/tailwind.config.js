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
  backgroundSize: {
   auto: "auto",
   cover: "cover",
   contain: "contain",
   "120%": "120%",
   "100%": "100%",
   16: "4rem",
  },
  container: {
   center: true,
   padding: "2rem",
   screens: {
    "2xl": "1400px",
   },
  },
  extend: {
   transitionProperty: {
    height: "height",
   },
   aspectRatio: {
    "3/4": "3 / 4",
   },
   colors: {
    border: "hsl(var(--border))",
    input: "hsl(var(--input))",
    ring: "hsl(var(--ring))",
    background: "#000",
    foreground: "#fff",
    primary: {
     DEFAULT: "#ffffff",
     foreground: "#000000",
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
     DEFAULT: "rgb(255 255 255)",
     foreground: "#000",
    },
    popover: {
     DEFAULT: "#111",
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
     from: {
      opacity: 0,
     },
     to: {
      opacity: 1,
     },
    },
    "fade-out": {
     to: {
      opacity: 0,
     },
    },
    "move-up": {
     "0%": {
      bottom: "-1vh",
      opacity: 0,
     },
     to: {
      opacity: 1,
     },
    },
    "move-down": {
     to: {
      bottom: "-1vh",
      opacity: 0,
     },
    },

    "color-change": {
     "0%": {
      background: "#ffffff00",
     },
     "100%": {},
    },
   },
   animation: {
    "expand-width": "expand-width 0.75s ease-in-out",
    "accordion-down": "accordion-down 0.2s ease-out",
    "accordion-up": "accordion-up 0.2s ease-out",
    fadein: "fade-in ease-in 0.2s",
    moveup: "move-up ease-in 0.2s",
    fadeout: "fade-out ease-out 0.2s",
    movedown: "move-down ease-out 0.2s",
    colorchange: "color-change ease-in",
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
