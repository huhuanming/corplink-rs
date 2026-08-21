#!/usr/bin/env node

'use strict';

const https = require('node:https');
const path = require('node:path');
const { spawn, spawnSync } = require('node:child_process');
const { version: currentVersion } = require('../package.json');

const latestPackageUrl = 'https://registry.npmjs.org/feilian-cli/latest';
const updateArguments = ['install', '--global', 'feilian-cli@latest'];

const packages = {
  'darwin-arm64': 'feilian-cli-darwin-arm64',
  'darwin-x64': 'feilian-cli-darwin-x64',
  'linux-arm64': 'feilian-cli-linux-arm64',
  'linux-x64': 'feilian-cli-linux-x64',
  'win32-x64': 'feilian-cli-windows-x64',
};

const target = `${process.platform}-${process.arch}`;
const packageName = packages[target];

if (!packageName) {
  console.error(`feilian-cli does not support ${target}`);
  process.exit(1);
}

const executableName = process.platform === 'win32' ? 'feilian-cli.exe' : 'feilian-cli';

function versionParts(version) {
  const parts = [];
  for (const part of version.replace(/^v/, '').split(/[.\-+]/)) {
    if (!/^\d+$/.test(part)) {
      break;
    }
    parts.push(Number.parseInt(part, 10));
  }
  return parts;
}

function compareVersions(left, right) {
  const leftParts = versionParts(left);
  const rightParts = versionParts(right);
  const length = Math.max(leftParts.length, rightParts.length);

  for (let index = 0; index < length; index += 1) {
    const difference = (leftParts[index] ?? 0) - (rightParts[index] ?? 0);
    if (difference !== 0) {
      return Math.sign(difference);
    }
  }
  return 0;
}

function fetchLatestVersion() {
  return new Promise((resolve, reject) => {
    const request = https.get(
      latestPackageUrl,
      {
        headers: {
          accept: 'application/json',
          'user-agent': `feilian-cli/${currentVersion}`,
        },
      },
      (response) => {
        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`npm returned HTTP ${response.statusCode}`));
          return;
        }

        response.setEncoding('utf8');
        let body = '';
        response.on('data', (chunk) => {
          body += chunk;
        });
        response.on('end', () => {
          try {
            const version = JSON.parse(body).version;
            if (typeof version !== 'string' || version.length === 0) {
              throw new Error('npm response is missing a version');
            }
            resolve(version);
          } catch (error) {
            reject(error);
          }
        });
      },
    );

    request.setTimeout(3000, () => request.destroy(new Error('npm update check timed out')));
    request.on('error', reject);
  });
}

function npmExecutable() {
  return process.platform === 'win32' ? 'npm.cmd' : 'npm';
}

function installLatestVersion() {
  return spawnSync(npmExecutable(), updateArguments, { stdio: 'inherit' });
}

function findGlobalLauncher() {
  const result = spawnSync(npmExecutable(), ['root', '--global'], {
    encoding: 'utf8',
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error('npm root --global failed');
  }
  return path.join(result.stdout.trim(), 'feilian-cli', 'bin', 'feilian-cli.js');
}

async function reportUpdateStatus() {
  const latestVersion = await fetchLatestVersion();
  if (compareVersions(latestVersion, currentVersion) > 0) {
    console.log(`feilian-cli update available: ${currentVersion} -> ${latestVersion}`);
    console.log('run: npm install --global feilian-cli@latest');
    return;
  }
  console.log(`feilian-cli ${currentVersion} is up to date`);
}

async function installUpdateIfAvailable(args, dependencies = {}) {
  const {
    fetchVersion = fetchLatestVersion,
    installVersion = installLatestVersion,
    locateLauncher = findGlobalLauncher,
    installedVersion = currentVersion,
    log = console.error,
  } = dependencies;

  if (args.includes('--check-update')) {
    return null;
  }

  let latestVersion;
  try {
    latestVersion = await fetchVersion();
  } catch (_error) {
    return null;
  }

  if (compareVersions(latestVersion, installedVersion) <= 0) {
    return null;
  }

  log(`feilian-cli update available: ${installedVersion} -> ${latestVersion}`);
  log('installing the latest version automatically...');
  const result = installVersion();
  if (result.error || result.status !== 0) {
    const reason = result.error ? `: ${result.error.message}` : '';
    log(`automatic update failed${reason}; continuing with ${installedVersion}`);
    return null;
  }

  log(`feilian-cli updated to ${latestVersion}; starting the new version...`);
  try {
    return locateLauncher();
  } catch (error) {
    log(`failed to locate the updated CLI: ${error.message}`);
    log('run feilian-cli again to use the installed version');
    return false;
  }
}

function startChild(command, args, env = process.env) {
  const child = spawn(command, args, { stdio: 'inherit', env });

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
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length === 1 && args[0] === '--check-update') {
    await reportUpdateStatus();
    return;
  }
  if (args.length === 1 && ['--version', '-V'].includes(args[0])) {
    console.log(`feilian-cli ${currentVersion}`);
    return;
  }

  const updatedLauncher = await installUpdateIfAvailable(args);
  if (updatedLauncher === false) {
    return;
  }
  if (updatedLauncher) {
    startChild(process.execPath, [updatedLauncher, ...args]);
    return;
  }

  let executable;
  try {
    executable = require.resolve(`${packageName}/bin/${executableName}`);
  } catch (_error) {
    console.error(`The native package ${packageName} is missing.`);
    console.error('Reinstall with: npm install --global feilian-cli@latest');
    process.exitCode = 1;
    return;
  }
  startChild(executable, args, {
    ...process.env,
    FEILIAN_CLI_NPM_LAUNCHER: '1',
  });
}

if (require.main === module) {
  main().catch((error) => {
    console.error(`failed to start feilian-cli: ${error.message}`);
    process.exitCode = 1;
  });
}

module.exports = { compareVersions, installUpdateIfAvailable };
