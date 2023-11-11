/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./static/**/*.{html,js}", "./templates/**/*.{html,js}"],
  theme: {
	  extend: {},
	  colors: {
		transparent: 'transparent',
		'dark-pink': '#E7CDC5',
		'light-pink': '#EDDAD4',
		'white': '#FFFFFF',
		'brown': '#433A3B',
	  },
  },
  plugins: [],
}

