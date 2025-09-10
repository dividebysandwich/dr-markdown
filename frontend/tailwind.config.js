/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  // The content scan is failing, so we use safelist to force classes to be generated.
  safelist: [
    'text-4xl', 'font-bold', 'text-lg', 'text-gray-900', 'dark:text-gray-200', 'text-gray-800', 'text-gray-600', 'text-gray-500', 'text-blue-600', 'hover:text-blue-800', 'underline',
    'min-h-screen', 'bg-gray-50', 'dark:bg-gray-900',
    'flex', 'items-center', 'justify-center', 'text-center',
    'px-4', 'py-2', 'text-sm', 'font-medium', 'font-semibold',
    'text-white', 'bg-blue-600', 'hover:bg-blue-700', 'bg-red-600', 'hover:bg-red-700', 'bg-gray-100', 'hover:bg-gray-200',
    'border', 'border-transparent', 'border-gray-300', 'rounded-md',
    'focus:outline-none', 'focus:ring-2', 'focus:ring-blue-500', 'focus:ring-red-500',
    'disabled:opacity-50',
    // Add any other specific classes you find are missing
  ],
  content: [
    "./index.html",
    "./src/**/*.rs"
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
