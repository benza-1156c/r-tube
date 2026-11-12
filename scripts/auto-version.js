import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

function getGitTags() {
  try {
    const stdout = execSync("git tag -l", { encoding: "utf8" });
    return stdout
      .split("\n")
      .map((t) => t.trim())
      .filter(Boolean);
  } catch {
    return [];
  }
}

function parseSemver(v) {
  if (!v) return null;
  const clean = v.replace(/^v/, "").trim();
  const match = clean.match(/^(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9.]+))?$/);
  if (!match) return null;
  return {
    raw: v,
    clean,
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
    prerelease: match[4] || null,
  };
}

function compareSemver(a, b) {
  if (a.major !== b.major) return a.major - b.major;
  if (a.minor !== b.minor) return a.minor - b.minor;
  if (a.patch !== b.patch) return a.patch - b.patch;
  return 0;
}

// 1. Parse CLI arguments & environment
const args = process.argv.slice(2);
let bumpArg = "auto";
let explicitVersion = "";

for (let i = 0; i < args.length; i++) {
  const arg = args[i];
  if (arg === "--bump" && args[i + 1]) {
    bumpArg = args[i + 1].toLowerCase().trim();
    i++;
  } else if (!arg.startsWith("-")) {
    const candidate = arg.trim();
    if (candidate && candidate !== "undefined" && candidate !== "null") {
      explicitVersion = candidate;
    }
  }
}

if (!explicitVersion && process.env.APP_VERSION && process.env.APP_VERSION !== "undefined" && process.env.APP_VERSION !== "null") {
  explicitVersion = process.env.APP_VERSION.trim();
}
if (!explicitVersion && process.env.TAG_NAME && process.env.TAG_NAME !== "undefined" && process.env.TAG_NAME !== "null") {
  explicitVersion = process.env.TAG_NAME.trim();
}

const pkgPath = path.resolve("package.json");
const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
const baseVersion = pkg.version || "0.1.0";
const parsedPkg = parseSemver(baseVersion);

let nextVersion = "";

if (explicitVersion) {
  nextVersion = explicitVersion.replace(/^v/, "").trim();
  console.log(`[Auto-Version] Using explicit version: ${nextVersion}`);
} else {
  // 2. Auto-detect from git tags & commits
  const tags = getGitTags();
  const semverTags = tags
    .map((t) => parseSemver(t))
    .filter((t) => t !== null)
    .sort((a, b) => compareSemver(a, b));

  if (semverTags.length === 0) {
    // No tags exist yet in git
    if (bumpArg === "major" && parsedPkg) {
      nextVersion = `${parsedPkg.major + 1}.0.0`;
    } else if (bumpArg === "minor" && parsedPkg) {
      nextVersion = `${parsedPkg.major}.${parsedPkg.minor + 1}.0`;
    } else if (bumpArg === "patch" && parsedPkg) {
      nextVersion = `${parsedPkg.major}.${parsedPkg.minor}.${parsedPkg.patch + 1}`;
    } else {
      nextVersion = baseVersion;
    }
  } else {
    const highestTag = semverTags[semverTags.length - 1];
    const latestTagRaw = highestTag.raw;

    // Check if package.json has already been manually bumped higher than git tag
    if (parsedPkg && compareSemver(parsedPkg, highestTag) > 0) {
      nextVersion = parsedPkg.clean;
      console.log(`[Auto-Version] Detected package.json version (${nextVersion}) is higher than git tag (${highestTag.clean})`);
    } else {
      let bumpType = bumpArg;

      if (bumpType === "auto") {
        try {
          const log = execSync(`git log ${latestTagRaw}..HEAD --oneline`, {
            encoding: "utf8",
          });
          if (log.includes("BREAKING CHANGE:") || log.includes("!:")) {
            bumpType = "major";
          } else if (
            log.toLowerCase().includes("feat:") ||
            log.toLowerCase().includes("feat(") ||
            log.toLowerCase().includes("feature:")
          ) {
            bumpType = "minor";
          } else {
            bumpType = "patch";
          }
        } catch {
          bumpType = "patch";
        }
      }

      console.log(`[Auto-Version] Latest tag is ${latestTagRaw}, applying bump: ${bumpType}`);

      if (bumpType === "major") {
        nextVersion = `${highestTag.major + 1}.0.0`;
      } else if (bumpType === "minor") {
        nextVersion = `${highestTag.major}.${highestTag.minor + 1}.0`;
      } else {
        nextVersion = `${highestTag.major}.${highestTag.minor}.${highestTag.patch + 1}`;
      }
    }
  }
}

const nextTag = `v${nextVersion}`;
console.log(`[Auto-Version] Final version: ${nextVersion} (Tag: ${nextTag})`);

// 3. Synchronize package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml
if (fs.existsSync(pkgPath)) {
  pkg.version = nextVersion;
  fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n", "utf8");
  console.log(`  ✓ Updated package.json -> ${nextVersion}`);
}

const tauriConfPath = path.resolve("src-tauri/tauri.conf.json");
if (fs.existsSync(tauriConfPath)) {
  const conf = JSON.parse(fs.readFileSync(tauriConfPath, "utf8"));
  conf.version = nextVersion;
  fs.writeFileSync(tauriConfPath, JSON.stringify(conf, null, 2) + "\n", "utf8");
  console.log(`  ✓ Updated src-tauri/tauri.conf.json -> ${nextVersion}`);
}

const cargoPath = path.resolve("src-tauri/Cargo.toml");
if (fs.existsSync(cargoPath)) {
  let cargo = fs.readFileSync(cargoPath, "utf8");
  cargo = cargo.replace(/^version\s*=\s*"[^"]+"/m, `version = "${nextVersion}"`);
  fs.writeFileSync(cargoPath, cargo, "utf8");
  console.log(`  ✓ Updated src-tauri/Cargo.toml -> ${nextVersion}`);
}

// 4. Export to $GITHUB_OUTPUT & $GITHUB_ENV if running inside GitHub Actions
const githubOutput = process.env.GITHUB_OUTPUT;
if (githubOutput) {
  fs.appendFileSync(githubOutput, `version=${nextVersion}\n`);
  fs.appendFileSync(githubOutput, `tag=${nextTag}\n`);
  console.log(`  ✓ Exported to $GITHUB_OUTPUT (version=${nextVersion}, tag=${nextTag})`);
}

const githubEnv = process.env.GITHUB_ENV;
if (githubEnv) {
  fs.appendFileSync(githubEnv, `APP_VERSION=${nextVersion}\n`);
  fs.appendFileSync(githubEnv, `APP_TAG=${nextTag}\n`);
}

console.log(`RELEASE_TAG=${nextTag}`);
console.log(`RELEASE_VERSION=${nextVersion}`);
