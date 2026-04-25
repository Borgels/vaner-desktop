#!/usr/bin/env node
// 0.8.6 WS-DESK-LINUX — predev / prebuild hook.
//
// Regenerates the ts-rs TypeScript bindings shipped by the Vaner
// `vaner-contract` crate and rsyncs them into `src/lib/contract/`.
// Skipped silently when the local Vaner workspace is unavailable
// (CI / fresh clones / users who only build the frontend).
//
// Configuration:
//   VANER_REPO  — path to the Vaner workspace root.
//                 Defaults to `../Vaner` relative to this repo.
//
// The hand-mirrored types in `src/lib/contract/setup-types.ts` are
// preserved by rsync because the ts-rs codegen does not currently
// emit `setup-types.ts` — when WS11 ships setup-type ts-rs annotations,
// remove the hand-mirror and let the rsync overwrite take effect.

import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(HERE, "..");
const TARGET_DIR = resolve(REPO_ROOT, "src/lib/contract");

const vanerRepo = resolve(
  process.env.VANER_REPO ?? resolve(REPO_ROOT, "..", "Vaner"),
);
const cargoToml = resolve(vanerRepo, "Cargo.toml");
const bindingsDir = resolve(vanerRepo, "crates/vaner-contract/bindings");

if (!existsSync(cargoToml)) {
  console.log(
    `[regen-contract-bindings] Vaner workspace not found at ${vanerRepo}; skipping ts-rs regen. ` +
      `Set VANER_REPO=<path> if your checkout lives elsewhere.`,
  );
  process.exit(0);
}

if (!hasOnPath("cargo")) {
  console.log(
    "[regen-contract-bindings] cargo not on PATH; skipping ts-rs regen. " +
      "The hand-mirrored types under src/lib/contract/ remain authoritative.",
  );
  process.exit(0);
}

if (!hasOnPath("rsync")) {
  console.log(
    "[regen-contract-bindings] rsync not on PATH; skipping bindings sync. " +
      "Hand-mirrored types under src/lib/contract/ remain authoritative.",
  );
  process.exit(0);
}

console.log(
  `[regen-contract-bindings] regenerating ts-rs bindings from ${vanerRepo}…`,
);

try {
  execSync(
    `cargo run --quiet --example export_bindings --features ts-rs --manifest-path "${cargoToml}" --package vaner-contract`,
    { stdio: "inherit" },
  );
} catch (err) {
  console.warn(
    "[regen-contract-bindings] cargo run failed; keeping existing bindings. " +
      "Hand-mirrored types under src/lib/contract/ remain authoritative.",
  );
  console.warn(`  reason: ${err?.message ?? err}`);
  process.exit(0);
}

if (!existsSync(bindingsDir)) {
  console.log(
    `[regen-contract-bindings] no bindings dir at ${bindingsDir} after cargo run; nothing to sync.`,
  );
  process.exit(0);
}

try {
  // rsync -a preserves timestamps; --include / --exclude keeps the
  // hand-mirrored setup-types.ts (until ts-rs setup annotations land
  // and the bindings dir starts emitting it). Add other hand-mirrored
  // files to the --exclude list as needed.
  execSync(
    `rsync -a --exclude=setup-types.ts "${bindingsDir}/" "${TARGET_DIR}/"`,
    { stdio: "inherit" },
  );
  console.log(
    `[regen-contract-bindings] synced bindings into ${TARGET_DIR}`,
  );
} catch (err) {
  console.warn(
    "[regen-contract-bindings] rsync failed; keeping existing bindings.",
  );
  console.warn(`  reason: ${err?.message ?? err}`);
  process.exit(0);
}

function hasOnPath(bin) {
  try {
    execSync(`command -v ${bin}`, { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}
