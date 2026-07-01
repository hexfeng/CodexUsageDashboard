import { readFile } from "node:fs/promises";

const expected = process.argv[2]?.replace(/^v/, "");

const packageJson = JSON.parse(await readFile("package.json", "utf8"));
const tauriConfig = JSON.parse(await readFile("src-tauri/tauri.conf.json", "utf8"));
const cargoToml = await readFile("src-tauri/Cargo.toml", "utf8");
const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

const versions = {
  "package.json": packageJson.version,
  "src-tauri/Cargo.toml": cargoVersion,
  "src-tauri/tauri.conf.json": tauriConfig.version,
};

const unique = new Set(Object.values(versions));
if (unique.size !== 1 || unique.has(undefined)) {
  console.error("Version mismatch:");
  for (const [file, version] of Object.entries(versions)) {
    console.error(`- ${file}: ${version ?? "missing"}`);
  }
  process.exit(1);
}

const version = [...unique][0];
if (expected && version !== expected) {
  console.error(`Version ${version} does not match expected tag ${expected}.`);
  process.exit(1);
}

console.log(`Version ${version} is consistent.`);
