/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./static/**/*.{html,js}", "./templates/**/*.{html,js}"],
  theme: {
	  extend: {
		  keyframes: {
			  shake: {
				
   				 '10%, 90%': {
   				     transform: 'translate3d(-2px, 0, 0)'
   				 },

   				 '20%, 80%': {
   				     transform: 'translate3d(4px, 0, 0)'
   				 },

   				 '30%, 50%, 70%': {
   				     transform: 'translate3d(-8px, 0, 0)'
   				 },

   				 '40%, 60%': {
   				     transform: 'translate3d(8px, 0, 0)'
   				 },
			  },
		  },
		  animation: {
			  shake: 'shake 0.42s cubic-bezier(.36, .07, .19, .97) both'
		  }
	  },
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

