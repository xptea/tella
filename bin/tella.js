#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const binName = process.platform === 'win32' ? 'tella.exe' : 'tella';
const binPath = path.join(__dirname, '..', 'target', 'release', binName);

// Check if Rust is installed
try {
  require('child_process').execSync('rustc --version', { stdio: 'pipe' });
} catch (e) {
  console.error('❌ Rust is not installed.');
  console.error('Please install Rust from https://rustup.rs/');
  process.exit(1);
}

// Build if not exists
if (!fs.existsSync(binPath)) {
  console.log('📦 Building Tella...');
  try {
    require('child_process').execSync('cargo build --release', {
      cwd: path.dirname(__dirname),
      stdio: 'inherit'
    });
  } catch (error) {
    console.error('❌ Build failed:', error.message);
    process.exit(1);
  }
}

// Spawn the binary
spawn(binPath, process.argv.slice(2), { stdio: 'inherit' });