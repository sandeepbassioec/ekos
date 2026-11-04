// Lets `node --test` run the TypeScript sources directly (Node type stripping):
// maps the NodeNext-style "./x.js" relative imports onto "./x.ts".
import { register } from "node:module";
register("./hooks.mjs", import.meta.url);
