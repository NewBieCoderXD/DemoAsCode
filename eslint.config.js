import { defineConfig, globalIgnores } from "eslint/config";
import tseslint from "typescript-eslint";
import sonarjs from "eslint-plugin-sonarjs";

export default defineConfig([
  globalIgnores([
    "dist/**",
    "node_modules/**",
    "results/**",
    "target/**",
    "test-results/**",
    "website/build/**",
    "website/.docusaurus/**",
  ]),

  // Spread because tseslint exports an array of configs
  ...tseslint.configs.recommended,

  sonarjs.configs.recommended,
  {
    rules: {
      "sonarjs/cognitive-complexity": "error",
    },
  },
]);
