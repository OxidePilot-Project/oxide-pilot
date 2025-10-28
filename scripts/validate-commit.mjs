#!/usr/bin/env node
/**
 * Commit message validation script
 * Enforces conventional commit format
 */

import { readFileSync } from 'fs';
import chalk from 'chalk';

const commitMsgFile = process.argv[2];
const commitMsg = readFileSync(commitMsgFile, 'utf-8').trim();

// Conventional commit pattern
const pattern = /^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?: .{1,100}/;

// Allow merge commits
if (commitMsg.startsWith('Merge')) {
  process.exit(0);
}

if (!pattern.test(commitMsg)) {
  console.log(chalk.red.bold('\n❌ Invalid commit message format!\n'));
  console.log(chalk.yellow('Commit message must follow Conventional Commits format:\n'));
  console.log(chalk.white('  <type>[optional scope]: <description>\n'));
  console.log(chalk.cyan('Valid types:'));
  console.log(chalk.white('  feat:     A new feature'));
  console.log(chalk.white('  fix:      A bug fix'));
  console.log(chalk.white('  docs:     Documentation changes'));
  console.log(chalk.white('  style:    Code style changes (formatting, etc)'));
  console.log(chalk.white('  refactor: Code refactoring'));
  console.log(chalk.white('  perf:     Performance improvements'));
  console.log(chalk.white('  test:     Adding or updating tests'));
  console.log(chalk.white('  build:    Build system changes'));
  console.log(chalk.white('  ci:       CI/CD changes'));
  console.log(chalk.white('  chore:    Other changes\n'));
  console.log(chalk.cyan('Examples:'));
  console.log(chalk.white('  feat: add user authentication'));
  console.log(chalk.white('  fix(api): resolve memory leak in guardian'));
  console.log(chalk.white('  docs: update README with installation steps\n'));
  console.log(chalk.yellow(`Your commit message:\n  ${chalk.red(commitMsg)}\n`));
  process.exit(1);
}

console.log(chalk.green('✓ Commit message format is valid\n'));
process.exit(0);
