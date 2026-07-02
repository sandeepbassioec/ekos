import { defineConfig } from 'vite'
import path from 'path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'


function figmaAssetResolver() {
  return {
    name: 'figma-asset-resolver',
    resolveId(id) {
      if (id.startsWith('figma:asset/')) {
        const filename = id.replace('figma:asset/', '')
        return path.resolve(__dirname, 'src/assets', filename)
      }
    },
  }
}

export default defineConfig({
  plugins: [
    figmaAssetResolver(),
    // The React and Tailwind plugins are both required for Make, even if
    // Tailwind is not being actively used – do not remove them
    react({
      fastRefresh: true,
      babel: {
        plugins: [],
      },
    }),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      // React is managed by pnpm virtual store — let Vite resolve it naturally
      'react': path.resolve(__dirname, 'node_modules/.pnpm/react@18.3.1/node_modules/react'),
      'react-dom': path.resolve(__dirname, 'node_modules/.pnpm/react-dom@18.3.1_react@18.3.1/node_modules/react-dom'),
    },
    // Force a single React copy — prevents "multiple React instances" from peer deps (e.g. @tiptap/react, @xyflow/react)
    dedupe: ['react', 'react-dom', 'react/jsx-runtime', 'yjs', 'y-protocols', 'lib0'],
  },

  // File types to support raw imports. Never add .css, .tsx, or .ts files to this.
  assetsInclude: ['**/*.svg', '**/*.csv'],
  
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      '@tiptap/react',
      '@tiptap/starter-kit',
      '@tiptap/extension-floating-menu',
      '@tiptap/extension-bubble-menu',
      '@tiptap/extension-placeholder',
      '@tiptap/extension-table',
      '@tiptap/extension-image',
      '@tiptap/suggestion',
      'mermaid',
      'yjs',
      'y-indexeddb',
      'y-websocket',
      'y-protocols/sync',
      'y-protocols/awareness',
      '@xyflow/react',
      'xlsx',
      'pptxgenjs',
      'lucide-react',
    ],
    exclude: ['@react-refresh', '@maxgraph/core'],
  },
  
  server: {
    hmr: {
      overlay: true,
    },
  },
  
  build: {
    sourcemap: false,
    commonjsOptions: {
      include: [/node_modules/],
    },
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor-react': ['react', 'react-dom'],
          'vendor-xlsx': ['xlsx'],
          'vendor-pptx': ['pptxgenjs'],
        },
      },
    },
  },
})