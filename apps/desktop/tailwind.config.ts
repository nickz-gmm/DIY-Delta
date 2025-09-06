import type { Config } from 'tailwindcss'
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: { bg:'#0b0d10', panel:'#121418', accent:'#69f0ae', mute:'#8a8f98' },
      borderRadius: { xl:'1rem', '2xl':'1.25rem' },
      boxShadow: { soft:'0 10px 30px rgba(0,0,0,0.35)' }
    }
  },
  plugins: []
} satisfies Config
