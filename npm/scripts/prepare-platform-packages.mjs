#!/usr/bin/env node

import { chmod, copyFile, mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, '../..');
const inputDirectory = path.resolve(process.argv[2] ?? 'release/native');
const outputDirectory = path.resolve(process.argv[3] ?? '.npm-release');
const metaPackage = JSON.parse(
  await readFile(path.join(repositoryRoot, 'npm/feilian-cli/package.json'), 'utf8'),
);

const targets = [
  { os: 'darwin', cpu: 'arm64', asset: 'feilian-cli-darwin-arm64' },
  { os: 'darwin', cpu: 'x64', asset: 'feilian-cli-darwin-x64' },
  { os: 'linux', cpu: 'arm64', asset: 'feilian-cli-linux-arm64' },
  { os: 'linux', cpu: 'x64', asset: 'feilian-cli-linux-x64' },
  { os: 'win32', cpu: 'x64', asset: 'feilian-cli-win32-x64.exe' },
];

await mkdir(outputDirectory, { recursive: true });

for (const target of targets) {
  const packageName = `feilian-cli-${target.os}-${target.cpu}`;
  const packageDirectory = path.join(outputDirectory, packageName);
  const binaryName = target.os === 'win32' ? 'feilian-cli.exe' : 'feilian-cli';
  const binaryDirectory = path.join(packageDirectory, 'bin');
  await mkdir(binaryDirectory, { recursive: true });
  await copyFile(path.join(inputDirectory, target.asset), path.join(binaryDirectory, binaryName));
  if (target.os !== 'win32') {
    await chmod(path.join(binaryDirectory, binaryName), 0o755);
  }
  await copyFile(path.join(repositoryRoot, 'license.txt'), path.join(packageDirectory, 'license.txt'));

  const packageJson = {
    name: packageName,
    version: metaPackage.version,
    description: `Native ${target.os}-${target.cpu} binary for feilian-cli.`,
    license: metaPackage.license,
    repository: metaPackage.repository,
    homepage: metaPackage.homepage,
    os: [target.os],
    cpu: [target.cpu],
    files: [`bin/${binaryName}`, 'README.md', 'license.txt'],
    publishConfig: { access: 'public' },
  };
  await writeFile(
    path.join(packageDirectory, 'package.json'),
    `${JSON.stringify(packageJson, null, 2)}\n`,
  );
  await writeFile(
    path.join(packageDirectory, 'README.md'),
    `# ${packageName}\n\nNative ${target.os}-${target.cpu} binary used by [feilian-cli](https://www.npmjs.com/package/feilian-cli). Install the main package instead of this package directly.\n`,
  );
}

console.log(`prepared ${targets.length} platform packages in ${outputDirectory}`);
