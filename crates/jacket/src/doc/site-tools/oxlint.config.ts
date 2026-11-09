import { defineConfig } from 'oxlint';

export default defineConfig({
  plugins: ['eslint', 'unicorn', 'oxc'],
  categories: {
    correctness: 'error',
    suspicious: 'error',
  },
  env: {
    browser: true,
    es2024: true,
  },
  ignorePatterns: ['node_modules/**'],
  rules: {
    'no-empty': ['error', { allowEmptyCatch: true }],
    'no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
    'unicorn/require-post-message-target-origin': 'off',
  },
});
