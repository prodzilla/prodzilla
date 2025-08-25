import { defineConfig } from '@rsbuild/core';
import { pluginReact } from '@rsbuild/plugin-react';
import path from 'path';

export default defineConfig({
  html: {
    title: 'Prodzilla',
  },
  plugins: [pluginReact()],
  source: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
  output: {
    assetPrefix: 'ui/',
  },
});
