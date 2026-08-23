import { existsSync, mkdirSync, readdirSync, statSync, copyFileSync } from "node:fs";
import { join } from "node:path";

const SRC = join("src", "profile_picker");
const DST = join("dist", "profile-picker");

const EXTENSIONS = new Set(["html", "css", "js", "json"]);

function copyDir(src, dst) {
  mkdirSync(dst, { recursive: true });
  for (const entry of readdirSync(src)) {
    const srcPath = join(src, entry);
    const dstPath = join(dst, entry);
    if (statSync(srcPath).isDirectory()) {
      copyDir(srcPath, dstPath);
    } else {
      const ext = entry.split(".").pop()?.toLowerCase() ?? "";
      if (EXTENSIONS.has(ext)) {
        copyFileSync(srcPath, dstPath);
      }
    }
  }
}

if (existsSync(SRC)) {
  copyDir(SRC, DST);
  console.log(`[build-frontend] copied ${SRC} -> ${DST}`);
} else {
  console.warn(`[build-frontend] source not found: ${SRC}`);
}
