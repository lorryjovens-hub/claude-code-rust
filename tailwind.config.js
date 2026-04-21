/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        claude: {
          bg: 'var(--bg-claude-main)',
          sidebar: 'var(--bg-claude-sidebar)',
          border: 'var(--border-claude)',
          text: 'var(--text-claude-main)',
          textSecondary: 'var(--text-claude-secondary)',
          accent: 'var(--bg-claude-accent)',
          hover: 'var(--bg-claude-hover)',
          btnHover: 'var(--bg-claude-btn-hover)',
          input: 'var(--bg-claude-input)',
          avatar: 'var(--bg-claude-avatar)',
          avatarText: 'var(--text-claude-avatar)',
        }
      },
      fontFamily: {
        sans: ['Figtree', 'sans-serif'],
        serif: ['Source Serif 4', 'serif'],
        system: ['system-ui', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', '"Open Sans"', '"Helvetica Neue"', 'sans-serif'],
      }
    }
  },
  plugins: [],
}
