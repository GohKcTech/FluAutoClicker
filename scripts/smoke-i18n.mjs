import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { cwd } from "node:process";

const langDir = resolve(cwd(), "src/lang");

const files = [
  "en.json", "ru.json", "de.json", "fr.json",
  "es.json", "zh.json", "pt.json", "ja.json", "ko.json",
];

const dicts = {};
for (const file of files) {
  const path = resolve(langDir, file);
  const raw = readFileSync(path, "utf8");
  try {
    dicts[file] = JSON.parse(raw);
  } catch (e) {
    console.error(`FAIL: ${file} is not valid JSON — ${e.message}`);
    process.exit(1);
  }
}

const ref = dicts["en.json"];
const refKeys = Object.keys(ref).sort();

let failed = false;

const emptyRefKeys = refKeys.filter((k) => ref[k] === "");
if (emptyRefKeys.length) {
  console.error(`FAIL: en.json has empty values for: ${emptyRefKeys.join(", ")}`);
  failed = true;
}

for (const file of files) {
  const dict = dicts[file];
  const keys = Object.keys(dict).sort();

  if (keys.length !== refKeys.length) {
    console.error(`FAIL: ${file} has ${keys.length} keys, expected ${refKeys.length}`);
    failed = true;
  }

  const missing = refKeys.filter((k) => !(k in dict));
  if (missing.length) {
    console.error(`FAIL: ${file} is missing keys: ${missing.join(", ")}`);
    failed = true;
  }

  const extra = keys.filter((k) => !(k in ref));
  if (extra.length) {
    console.error(`FAIL: ${file} has extra keys not in en.json: ${extra.join(", ")}`);
    failed = true;
  }
}

if (failed) {
  console.error("\ni18n smoke check failed.");
  process.exit(1);
}

console.log(`i18n smoke check passed. ${files.length} languages, ${refKeys.length} keys each.`);
