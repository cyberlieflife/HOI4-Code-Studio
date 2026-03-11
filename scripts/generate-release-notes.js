#!/usr/bin/env node

import fs from 'fs';
import os from 'os';
import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import ts from 'typescript';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '..');
const CHANGELOG_FILE = path.join(ROOT_DIR, 'src', 'data', 'changelog.ts');

function readArgs(argv) {
  const options = {};

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (!arg.startsWith('--')) {
      continue;
    }

    const key = arg.slice(2);
    const value = argv[index + 1];
    if (!value || value.startsWith('--')) {
      options[key] = true;
      continue;
    }

    options[key] = value;
    index += 1;
  }

  return options;
}

function parseTagToVersion(tag) {
  if (tag.startsWith('Dev_')) {
    return `v${tag.slice(4).replace(/_/g, '.')}-dev`;
  }

  if (tag.startsWith('v')) {
    return `v${tag.slice(1).replace(/_/g, '.')}`;
  }

  throw new Error(`无法识别的标签格式: ${tag}`);
}

async function loadChangelog() {
  const source = fs.readFileSync(CHANGELOG_FILE, 'utf8');
  const transpiled = ts.transpileModule(source, {
    compilerOptions: {
      module: ts.ModuleKind.ES2020,
      target: ts.ScriptTarget.ES2020
    }
  }).outputText;

  const tempFile = path.join(
    os.tmpdir(),
    `hoi4-release-notes-${Date.now()}-${Math.random().toString(16).slice(2)}.mjs`
  );

  fs.writeFileSync(tempFile, transpiled, 'utf8');

  try {
    const module = await import(pathToFileURL(tempFile).href);
    if (!Array.isArray(module.changelog)) {
      throw new Error('changelog.ts 未导出有效的 changelog 数组');
    }

    return module.changelog;
  } finally {
    fs.rmSync(tempFile, { force: true });
  }
}

function formatReleaseNotes(entry) {
  const lines = ['## 更新内容'];

  if (entry.description) {
    lines.push(`- 概述：${entry.description}`);
  }

  for (const change of entry.changes) {
    lines.push(`- ${change.content}`);
  }

  return `${lines.join('\n')}\n`;
}

function writeGithubOutput(filePath, values) {
  if (!filePath) {
    return;
  }

  const lines = Object.entries(values).map(([key, value]) => `${key}=${value}`);
  fs.appendFileSync(filePath, `${lines.join('\n')}\n`, 'utf8');
}

async function main() {
  const args = readArgs(process.argv.slice(2));
  const version = args.version || (args.tag ? parseTagToVersion(args.tag) : null);

  if (!version) {
    throw new Error('请通过 --version 或 --tag 指定目标版本');
  }

  const changelog = await loadChangelog();
  const entry = changelog.find((item) => item.version === version);

  if (!entry) {
    throw new Error(`changelog.ts 中未找到版本 ${version} 的发布记录`);
  }

  const body = formatReleaseNotes(entry);
  const releaseName = entry.version;

  if (args.output) {
    fs.writeFileSync(path.resolve(ROOT_DIR, args.output), body, 'utf8');
  } else {
    process.stdout.write(body);
  }

  writeGithubOutput(args['github-output'], {
    version: entry.version,
    release_name: releaseName
  });
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
