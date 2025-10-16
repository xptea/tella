#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('🔧 Setting up Tella globally...\n');

// Get npm prefix
let prefix;
try {
  prefix = execSync('npm config get prefix', { encoding: 'utf8' }).trim();
} catch (e) {
  console.error('❌ Could not get npm prefix');
  process.exit(1);
}

const binDir = path.join(prefix, 'bin');
const nodeModulesDir = path.join(prefix, 'node_modules');
const tellaScript = path.join(nodeModulesDir, 'tella', 'bin', 'tella.js');
const tellaCmd = path.join(binDir, 'tella.cmd');

// Create bin directory if it doesn't exist
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

// Create tella.cmd on Windows
if (process.platform === 'win32') {
  const cmdContent = `@ECHO off\r\nnode "${tellaScript}" %*\r\n`;
  fs.writeFileSync(tellaCmd, cmdContent, 'utf8');
  console.log('✅ Created tella.cmd in', binDir);
}

// Check if bin directory is in PATH
const pathEnv = process.env.PATH || '';
const pathDirs = pathEnv.split(path.delimiter);
const binInPath = pathDirs.some(dir => dir === binDir);

if (!binInPath) {
  console.log('\n⚠️  Adding npm global bin directory to PATH...');
  if (process.platform === 'win32') {
    try {
      execSync(`powershell -Command "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';${binDir}', 'User')"`, { stdio: 'inherit' });
      console.log('✅ Added to user PATH');
    } catch (e) {
      console.log('❌ Could not add to PATH automatically. Please run:');
      console.log(`[Environment]::SetEnvironmentVariable("Path", [Environment]::GetEnvironmentVariable("Path", "User") + ";${binDir}", "User")`);
    }
  } else {
    console.log(`Add this to your shell profile: export PATH="$PATH:${binDir}"`);
  }
} else {
  console.log('✅ PATH is configured correctly');
}

console.log('\n✅ Tella setup complete! Run: tella --help');