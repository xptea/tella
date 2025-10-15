#!/usr/bin/env node

/**
 * Tella Post-Install Script
 * Runs after NPM installation to set up the Rust binary
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('🔧 Setting up Tella CLI...\n');

try {
  // Check if Rust is installed
  try {
    execSync('rustc --version', { stdio: 'pipe' });
  } catch (e) {
    console.error('❌ Rust is not installed.');
    console.error('Please install Rust from https://rustup.rs/');
    process.exit(1);
  }

  console.log('✅ Rust detected');
  
  // Build the binary
  console.log('📦 Building Tella...');
  execSync('cargo build --release', { 
    cwd: __dirname,
    stdio: 'inherit'
  });

  console.log('\n✅ Tella installed successfully!');
  console.log('\n📝 Next steps:');
  console.log('1. Run: tella --settings');
  console.log('2. Add your Cerebras API key');
  console.log('3. Start using: tella your question here\n');

} catch (error) {
  console.error('❌ Installation failed:', error.message);
  process.exit(1);
}
