import { execFileSync, execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

/**
 * Decide the next app version and sync it into every manifest.
 *
 * Usage:
 *   bun run scripts/auto-version.js                # smart bump from commits
 *   bun run scripts/auto-version.js --bump patch   # force patch/minor/major
 *   bun run scripts/auto-version.js 1.2.3          # exact version
 *   bun run scripts/auto-version.js --current      # keep package.json version
 *
 * With `--bump auto` (the default) Conventional Commits since the last tag
 * decide the bump:
 *   type!: / BREAKING CHANGE -> major
 *   feat:                    -> minor
 *   fix:/perf:/refactor:/... -> patch
 *   chore:/docs:/style:/test:/ci:/build: only -> skip (no release)
 *
 * Chore-only changes (including Dependabot updates) therefore no longer bump
 * the version on their own. Explicit `--bump` and exact versions always win.
 */

const RELEASE_COMMIT = /^chore\(release\)/i;
const MERGE_COMMIT = /^merge\b/i;
const BREAKING_HEADER = /^[a-zA-Z]+(\([^)]*\))?!:/;
const BREAKING_FOOTER = /(^|\n)BREAKING[ -]CHANGE:/;
const FEATURE_HEADER = /^feat(\([^)]*\))?:/i;
const NON_RELEASING_HEADER =
  /^(chore|docs|style|test|ci|build)(\([^)]*\))?:/i;

function git(args, options = {}) {
  return execFileSync("git", args, { encoding: "utf8", ...options });
}

function getCommitMessages(sinceTag) {
  try {
    return git(["log", `${sinceTag}..HEAD`, "--format=%B%x00"])
      .split("\u0000")
      .map((message) => message.trim())
      .filter(Boolean);
  } catch {
    return [];
  }
}

/** Returns "major" | "minor" | "patch" | null (null = nothing to release). */
function detectBumpType(sinceTag) {
  const messages = getCommitMessages(sinceTag);
  if (messages.length === 0) return null;

  let bump = null;

  for (const message of messages) {
    const [header = "", ...rest] = message.split("\n");
    const body = rest.join("\n");

    if (RELEASE_COMMIT.test(header) || MERGE_COMMIT.test(header)) continue;

    if (BREAKING_HEADER.test(header) || BREAKING_FOOTER.test(body)) {
      return "major";
    }
    if (FEATURE_HEADER.test(header)) {
      bump = "minor";
      continue;
    }
    if (NON_RELEASING_HEADER.test(header)) continue;

    // fix:, perf:, refactor:, revert: and any non-conventional message.
    if (bump === null) bump = "patch";
  }

  return bump;
}

function getSemverTags() {
  try {
    return execSync("git tag -l", { encoding: "utf8" })
      .split("\n")
      .map((tag) => tag.trim())
      .filter(Boolean)
      .map(parseSemver)
      .filter(Boolean)
      .sort(compareSemver);
  } catch {
    return [];
  }
}

function parseSemver(value) {
  if (!value) return null;
  const clean = value.replace(/^v/, "").trim();
  const match = clean.match(/^(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9.]+))?$/);
  if (!match) return null;

  return {
    raw: value,
    clean,
    major: Number.parseInt(match[1], 10),
    minor: Number.parseInt(match[2], 10),
    patch: Number.parseInt(match[3], 10),
    prerelease: match[4] || null,
  };
}

function compareSemver(a, b) {
  if (a.major !== b.major) return a.major - b.major;
  if (a.minor !== b.minor) return a.minor - b.minor;
  return a.patch - b.patch;
}

function bumpVersion(version, type) {
  if (type === "major") return `${version.major + 1}.0.0`;
  if (type === "minor") return `${version.major}.${version.minor + 1}.0`;
  return `${version.major}.${version.minor}.${version.patch + 1}`;
}

function setOutput(name, value) {
  console.log(`${name}=${value}`);
  if (process.env.GITHUB_OUTPUT) {
    fs.appendFileSync(process.env.GITHUB_OUTPUT, `${name}=${value}\n`);
  }
  if (process.env.GITHUB_ENV) {
    fs.appendFileSync(process.env.GITHUB_ENV, `${name.toUpperCase()}=${value}\n`);
  }
}

// ── CLI arguments ──────────────────────────────────────────────────────────
const args = process.argv.slice(2);
let bumpArg = "auto";
let explicitVersion = "";
let currentOnly = false;

for (let i = 0; i < args.length; i++) {
  const arg = args[i];

  if (arg === "--current") {
    currentOnly = true;
  } else if (arg === "--bump" && args[i + 1]) {
    bumpArg = args[i + 1].toLowerCase().trim();
    i++;
  } else if (!arg.startsWith("-")) {
    const candidate = arg.trim();
    if (candidate && candidate !== "undefined" && candidate !== "null") {
      explicitVersion = candidate;
    }
  }
}

for (const envValue of [process.env.APP_VERSION, process.env.TAG_NAME]) {
  if (!explicitVersion && envValue && envValue !== "undefined") {
    explicitVersion = envValue.trim();
  }
}

if (!["auto", "patch", "minor", "major"].includes(bumpArg)) {
  console.error(`[Auto-Version] Unknown bump type: ${bumpArg}`);
  process.exit(1);
}

const pkgPath = path.resolve("package.json");
const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
const parsedPkg = parseSemver(pkg.version || "0.1.0");

const tags = getSemverTags();
const highestTag = tags.length > 0 ? tags[tags.length - 1] : null;

let nextVersion = "";
let bumpType = null;

if (explicitVersion) {
  nextVersion = explicitVersion.replace(/^v/, "").trim();
  console.log(`[Auto-Version] Using explicit version: ${nextVersion}`);
} else if (currentOnly) {
  nextVersion = parsedPkg.clean;
  console.log(`[Auto-Version] Keeping current version: ${nextVersion}`);
} else if (!highestTag) {
  // No release yet: only an explicit bump moves the version, otherwise tag
  // whatever package.json already says.
  if (bumpArg === "auto") {
    nextVersion = parsedPkg.clean;
    console.log(`[Auto-Version] No tags yet, using ${nextVersion}`);
  } else {
    bumpType = bumpArg;
    nextVersion = bumpVersion(parsedPkg, bumpArg);
  }
} else if (parsedPkg.clean !== highestTag.clean && compareSemver(parsedPkg, highestTag) > 0) {
  // Someone bumped package.json by hand past the latest tag; respect it.
  nextVersion = parsedPkg.clean;
  console.log(
    `[Auto-Version] package.json (${nextVersion}) is ahead of ${highestTag.raw}; using it`,
  );
} else {
  bumpType = bumpArg === "auto" ? detectBumpType(highestTag.raw) : bumpArg;

  if (bumpType === null) {
    console.log(
      `[Auto-Version] No release-worthy commits since ${highestTag.raw} (only chore/docs/etc).`,
    );
    setOutput("skip", "true");
    setOutput("version", highestTag.clean);
    setOutput("tag", highestTag.raw);
    process.exit(0);
  }

  console.log(
    `[Auto-Version] ${highestTag.raw} + ${bumpType} bump${
      bumpArg === "auto" ? " (from Conventional Commits)" : ""
    }`,
  );
  nextVersion = bumpVersion(highestTag, bumpType);
}

if (!parseSemver(nextVersion)) {
  console.error(`[Auto-Version] Invalid version: ${nextVersion}`);
  process.exit(1);
}

const nextTag = nextVersion.startsWith("v") ? nextVersion : `v${nextVersion}`;
console.log(`[Auto-Version] Final version: ${nextVersion} (tag: ${nextTag})`);

// ── Sync manifests ─────────────────────────────────────────────────────────
pkg.version = nextVersion;
fs.writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`, "utf8");
console.log(`  ✓ package.json -> ${nextVersion}`);

const tauriConfPath = path.resolve("src-tauri/tauri.conf.json");
if (fs.existsSync(tauriConfPath)) {
  const conf = JSON.parse(fs.readFileSync(tauriConfPath, "utf8"));
  conf.version = nextVersion;
  fs.writeFileSync(tauriConfPath, `${JSON.stringify(conf, null, 2)}\n`, "utf8");
  console.log(`  ✓ src-tauri/tauri.conf.json -> ${nextVersion}`);
}

const cargoPath = path.resolve("src-tauri/Cargo.toml");
if (fs.existsSync(cargoPath)) {
  const cargo = fs
    .readFileSync(cargoPath, "utf8")
    .replace(/^version\s*=\s*"[^"]+"/m, `version = "${nextVersion}"`);
  fs.writeFileSync(cargoPath, cargo, "utf8");
  console.log(`  ✓ src-tauri/Cargo.toml -> ${nextVersion}`);
}

const cargoLockPath = path.resolve("src-tauri/Cargo.lock");
if (fs.existsSync(cargoLockPath)) {
  const lock = fs
    .readFileSync(cargoLockPath, "utf8")
    .replace(/(name = "r-tube"\nversion = ")[^"]+(")/, `$1${nextVersion}$2`);
  fs.writeFileSync(cargoLockPath, lock, "utf8");
  console.log(`  ✓ src-tauri/Cargo.lock -> ${nextVersion}`);
}

setOutput("skip", "false");
setOutput("version", nextVersion);
setOutput("tag", nextTag);
