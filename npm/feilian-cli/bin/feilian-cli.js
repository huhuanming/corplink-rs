#!/usr/bin/env node

'use strict';

const { spawn } = require('node:child_process');

const packages = {
  'darwin-arm64': 'feilian-cli-darwin-arm64',
  'darwin-x64': 'feilian-cli-darwin-x64',
  'linux-arm64': 'feilian-cli-linux-arm64',
  'linux-x64': 'feilian-cli-linux-x64',
  'win32-x64': 'feilian-cli-win32-x64',
};

const target = `${process.platform}-${process.arch}`;
const packageName = packages[target];

if (!packageName) {
  console.error(`feilian-cli does not support ${target}`);
  process.exit(1);
}

const executableName = process.platform === 'win32' ? 'feilian-cli.exe' : 'feilian-cli';
let executable;

try {
  executable = require.resolve(`${packageName}/bin/${executableName}`);
} catch (error) {
  console.error(`The native package ${packageName} is missing.`);
  console.error('Reinstall with: npm install --global feilian-cli@latest');
  process.exit(1);
}

const child = spawn(executable, process.argv.slice(2), { stdio: 'inherit' });

for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => child.kill(signal));
}

child.on('error', (error) => {
  console.error(`failed to start feilian-cli: ${error.message}`);
  process.exitCode = 1;
});

child.on('exit', (code, signal) => {
  if (signal) {
    process.removeAllListeners(signal);
    process.kill(process.pid, signal);
    return;
  }
  process.exitCode = code ?? 1;
});
