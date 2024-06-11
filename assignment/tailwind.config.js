/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    fontFamily: {
      mono: [
        "DejaVu Sans Mono",
        "ui-monospace",
        "Cascadia Code",
        "Menlo",
        "Source Code Pro",
        "Consolas",
        "monospace",
      ],
    },
    extend: {},
  },
  plugins: [],
};
