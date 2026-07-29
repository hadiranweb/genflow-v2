import type { Config } from "tailwindcss";

export default {
  content: [
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/**/*.{js,ts,jsx,tsx,mdx}",
    "../../packages/ui/src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        teal: {
          50: "#f0fdfa",
          100: "#ccfbf1",
          200: "#99f6e4",
          300: "#5eead4",
          400: "#2dd4bf",
          500: "#298581",
          600: "#1a6b62",
          700: "#11544d",
          800: "#0d3d38",
          900: "#082824",
        },
        navy: {
          800: "#181c34",
          900: "#0f1228",
          950: "#080a18",
        },
      },
      fontFamily: {
        sans: ["Vazirmatn", "system-ui", "sans-serif"],
      },
      direction: {
        rtl: "rtl",
      },
    },
  },
  plugins: [],
} satisfies Config;
