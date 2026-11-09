import { defineConfig } from 'oxlint';

export default defineConfig({
  plugins: ['eslint', 'typescript', 'unicorn', 'oxc'],
  categories: {
    correctness: 'error',
    suspicious: 'error',
  },
  env: {
    builtin: true,
    es2024: true,
    node: true,
  },
  ignorePatterns: ['node_modules/**'],
  rules: {
    'no-empty': ['error', { allowEmptyCatch: true }],
    'no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
    // Asks for a target origin, which a worker's `parentPort.postMessage` does not have.
    'unicorn/require-post-message-target-origin': 'off',
  },
});
