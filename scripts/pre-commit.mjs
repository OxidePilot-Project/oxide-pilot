#!/usr/bin/env node
/**
 * Pre-commit validation script for Oxide Pilot
 * Ensures code quality before allowing commits
 */

import { execSync } from 'child_process';
import chalk from 'chalk';

const CHECKS = [
  {
    name: 'Rust Formatting',
    command: 'cargo fmt --all -- --check',
    fix: 'cargo fmt --all',
    critical: true,
  },
  {
    name: 'Clippy Linting',
    command: 'cargo clippy --workspace --features surrealdb-metrics -- -D warnings',
    fix: 'cargo clippy --fix --allow-dirty --allow-staged --workspace --features surrealdb-metrics',
    critical: true,
  },
  {
    name: 'Unit Tests',
    command: 'cargo test --lib --workspace --features surrealdb-metrics',
    critical: true,
  },
  {
    name: 'Build Check',
    command: 'cargo check --workspace --features surrealdb-metrics',
    critical: true,
  },
  {
    name: 'Frontend Linting',
    command: 'cd src-frontend && npm run check',
    critical: false,
  },
];

console.log(chalk.cyan.bold('\n🔍 Running pre-commit validation...\n'));

let hasErrors = false;
let hasFixes = false;
const results = [];

for (const check of CHECKS) {
  process.stdout.write(chalk.blue(`▶ ${check.name}... `));

  try {
    execSync(check.command, {
      stdio: 'pipe',
      encoding: 'utf-8',
    });
    console.log(chalk.green('✓'));
    results.push({ name: check.name, status: 'passed' });
  } catch (error) {
    console.log(chalk.red('✗'));
    results.push({
      name: check.name,
      status: 'failed',
      error: error.stderr || error.stdout,
    });

    if (check.critical) {
      hasErrors = true;
    }

    if (check.fix) {
      console.log(chalk.yellow(`  ℹ Auto-fix available: ${check.fix}`));
      hasFixes = true;
    }
  }
}

console.log('\n' + chalk.cyan('─'.repeat(60)) + '\n');

// Print summary
console.log(chalk.bold('📊 Validation Summary:\n'));
for (const result of results) {
  const icon = result.status === 'passed' ? chalk.green('✓') : chalk.red('✗');
  console.log(`${icon} ${result.name}`);

  if (result.error && result.status === 'failed') {
    const errorLines = result.error.split('\n').slice(0, 5);
    errorLines.forEach(line => {
      if (line.trim()) {
        console.log(chalk.gray(`  ${line}`));
      }
    });
  }
}

console.log('\n' + chalk.cyan('─'.repeat(60)) + '\n');

if (hasErrors) {
  console.log(chalk.red.bold('❌ Commit blocked due to validation errors!\n'));

  if (hasFixes) {
    console.log(chalk.yellow('💡 Some issues can be auto-fixed. Run:\n'));
    console.log(chalk.white('   cargo fmt --all'));
    console.log(chalk.white('   cargo clippy --fix --allow-dirty --allow-staged --workspace --features surrealdb-metrics\n'));
  }

  console.log(chalk.yellow('📝 Fix the issues above and try again.\n'));
  process.exit(1);
} else {
  console.log(chalk.green.bold('✅ All checks passed! Proceeding with commit.\n'));
  process.exit(0);
}
