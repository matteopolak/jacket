import { defineConfig } from "oxlint";

export default defineConfig({
  plugins: ["eslint", "typescript", "unicorn", "oxc"],
  categories: {
    correctness: "error",
    suspicious: "error",
  },
  env: {
    builtin: true,
    es2024: true,
    node: true,
  },
  ignorePatterns: ["dist/**", "out/**", "node_modules/**", ".vscode-test/**"],
  rules: {
    "no-empty": ["error", { allowEmptyCatch: true }],
    "no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
  },
});
