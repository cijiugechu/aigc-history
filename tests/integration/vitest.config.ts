import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["tests/integration/**/*.{test,spec}.ts"],
    testTimeout: 60000,
    hookTimeout: 60000,
    retry: 0,
    globals: true
  }
});
