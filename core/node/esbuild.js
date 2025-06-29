const esbuild = require("esbuild");

esbuild
  .build({
    entryPoints: ["index.js"],
    bundle: true,
    outfile: "dist/build.js",
    format: "cjs",
    minify: true,
    platform: "node",
  })
  .catch(() => process.exit(1));
