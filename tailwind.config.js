/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: "#007AFF",
          light: "#E8F0FE",
          dark: "#0051D5",
        },
      },
      fontFamily: {
        sans: ["-apple-system", "BlinkMacSystemFont", "SF Pro Text", "Helvetica Neue", "Segoe UI", "system-ui", "sans-serif"],
      },
    },
  },
  plugins: [],
};
