// Build the matching standalone helper before Tauri starts its development window.
import console from "node:console";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { createServer } from "vite";
const result = spawnSync("cargo", ["build", "--manifest-path", "src-tauri/Cargo.toml", "--bin", "bloodborne-overlay"], { stdio: "inherit" });
if (result.error || result.status !== 0) {
  if (result.error) console.error(result.error.message);
  process.exit(result.status ?? 1);
}
const server = await createServer();
await server.listen();
server.printUrls();
for (const signal of ["SIGINT", "SIGTERM"]) process.once(signal, async () => { await server.close(); process.exit(0); });
