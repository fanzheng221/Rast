const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const DEFAULT_FILE_COUNT = 10_000;
const DEFAULT_RULE_COUNT = 10;
const DEFAULT_ROUNDS = 1;

function parseArgs(argv) {
  const options = {
    files: DEFAULT_FILE_COUNT,
    rules: DEFAULT_RULE_COUNT,
    rounds: DEFAULT_ROUNDS,
    keepFixture: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--') {
      continue;
    }
    if (arg === '--files') {
      options.files = Number(argv[++i]);
    } else if (arg === '--rules') {
      options.rules = Number(argv[++i]);
    } else if (arg === '--rounds') {
      options.rounds = Number(argv[++i]);
    } else if (arg === '--keep-fixture') {
      options.keepFixture = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!Number.isInteger(options.files) || options.files <= 0) {
    throw new Error('--files must be a positive integer');
  }
  if (!Number.isInteger(options.rules) || options.rules <= 0 || options.rules > 10) {
    throw new Error('--rules must be an integer between 1 and 10');
  }
  if (!Number.isInteger(options.rounds) || options.rounds <= 0) {
    throw new Error('--rounds must be a positive integer');
  }

  return options;
}

function runCommand(command, args, cwd) {
  const start = process.hrtime.bigint();
  const result = spawnSync(command, args, {
    cwd,
    encoding: 'utf8',
    stdio: ['ignore', 'ignore', 'pipe'],
  });
  const elapsedMs = Number(process.hrtime.bigint() - start) / 1_000_000;

  if (result.status !== 0) {
    throw new Error(
      [
        `Command failed: ${command} ${args.join(' ')}`,
        `Exit code: ${result.status}`,
        result.error ? `error: ${result.error.message}` : '',
        result.stderr?.trim() ? `stderr: ${result.stderr.trim()}` : '',
      ]
        .filter(Boolean)
        .join('\n')
    );
  }

  return { elapsedMs };
}

function resolveAstGrepRunner(cwd) {
  const candidates = [
    { command: 'sg', baseArgs: [] },
    { command: 'ast-grep', baseArgs: [] },
    { command: 'pnpm', baseArgs: ['--package=@ast-grep/cli', 'dlx', 'sg'] },
  ];

  for (const candidate of candidates) {
    const probe = spawnSync(candidate.command, [...candidate.baseArgs, '--version'], {
      cwd,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    if (probe.status === 0) {
      return candidate;
    }
  }
  throw new Error(
    'Unable to find ast-grep executable. Install sg, or ensure pnpm can run @ast-grep/cli.'
  );
}

function buildRuleDefinitions() {
  return [
    { id: 'no-console', pattern: 'console.log($A)', fix: 'logger.info($A)' },
    { id: 'ternary-assignment', pattern: 'const $A = $B ? $C : $D;' },
    { id: 'guard-if', pattern: 'if ($A && $B) { $$$C }' },
    { id: 'map-transform', pattern: 'const $A = [1, 2, 3].map(($B) => $C);' },
    { id: 'work-call', pattern: 'doWork($A, $B);' },
    { id: 'fallback-call', pattern: 'const $A = fallback($B);' },
    { id: 'string-concat', pattern: 'const $A = "prefix-" + $B;' },
    { id: 'object-shape', pattern: 'const $A = { value, list, picked, message };' },
    { id: 'arrow-call', pattern: 'const $A = ($B) => doWork($B, $C);' },
    { id: 'function-shape', pattern: 'function $A($$$B) { $$$C }' },
  ];
}

function ensureDirectory(targetPath) {
  fs.mkdirSync(targetPath, { recursive: true });
}

function createFixtureProject(rootDir, fileCount) {
  const srcRoot = path.join(rootDir, 'src');
  ensureDirectory(srcRoot);

  for (let i = 0; i < fileCount; i += 1) {
    const ext = i % 2 === 0 ? 'ts' : 'js';
    const group = `group-${i % 100}`;
    const folder = path.join(srcRoot, group);
    ensureDirectory(folder);

    const code = [
      `function worker${i}(input, flag) {`,
      '  const value = input && input.value ? input.value : 0;',
      '  const list = [1, 2, 3].map((item) => item + value);',
      '  if (input && flag) {',
      '    console.log(value);',
      '  }',
      '  doWork(list, value);',
      '  const picked = value ? value : fallback(value);',
      '  const message = "prefix-" + value;',
      '  const composed = (payload) => doWork(payload, value);',
      '  const summary = { value, list, picked, message };',
      '  return summary;',
      '}',
      '',
      `worker${i}({ value: ${i} }, true);`,
      '',
    ].join('\n');

    fs.writeFileSync(path.join(folder, `file-${i}.${ext}`), code, 'utf8');
  }
}

function writeRuleFiles(rulesDir, selectedRules) {
  ensureDirectory(rulesDir);
  return selectedRules.map((rule, index) => {
    const patternBlock = rule.pattern
      .split('\n')
      .map((line) => `    ${line}`)
      .join('\n');
    const fixBlock = rule.fix
      ? rule.fix
          .split('\n')
          .map((line) => `  ${line}`)
          .join('\n')
      : '';
    const ruleText = [
      `id: ${rule.id}-${index + 1}`,
      'language: ts',
      'rule:',
      '  pattern: |',
      patternBlock,
      rule.fix ? 'fix: |' : '',
      fixBlock,
      '',
    ]
      .filter(Boolean)
      .join('\n');
    const rulePath = path.join(rulesDir, `rule-${index + 1}.yml`);
    fs.writeFileSync(rulePath, ruleText, 'utf8');
    return rulePath;
  });
}

function benchmarkRast(rastBinaryPath, fixtureDir, rulePaths, rounds, cwd) {
  const roundTimes = [];
  for (let round = 0; round < rounds; round += 1) {
    let total = 0;
    for (const rulePath of rulePaths) {
      const { elapsedMs } = runCommand(
        rastBinaryPath,
        ['scan', fixtureDir, rulePath, '--dry-run', '--output', 'json'],
        cwd
      );
      total += elapsedMs;
    }
    roundTimes.push(total);
  }
  return roundTimes;
}

function benchmarkAstGrep(astGrepRunner, fixtureDir, rulePaths, rounds, cwd) {
  const roundTimes = [];
  for (let round = 0; round < rounds; round += 1) {
    let total = 0;
    for (const rulePath of rulePaths) {
      const args = [
        ...astGrepRunner.baseArgs,
        'scan',
        fixtureDir,
        '--rule',
        rulePath,
        '--json=stream',
        '--color=never',
      ];
      const { elapsedMs } = runCommand(astGrepRunner.command, args, cwd);
      total += elapsedMs;
    }
    roundTimes.push(total);
  }
  return roundTimes;
}

function avg(values) {
  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

function formatMs(value) {
  return `${value.toFixed(2)} ms`;
}

function printRoundTable(rastTimes, astTimes) {
  console.log('\n| Tool | Round | Total Time |');
  console.log('| --- | ---: | ---: |');
  for (let i = 0; i < rastTimes.length; i += 1) {
    console.log(`| rast scan | ${i + 1} | ${formatMs(rastTimes[i])} |`);
    console.log(`| ast-grep | ${i + 1} | ${formatMs(astTimes[i])} |`);
  }
}

function printSummary(rastTimes, astTimes, fileCount, ruleCount, fixtureDir) {
  const rastAvg = avg(rastTimes);
  const astAvg = avg(astTimes);
  const speedup = astAvg / rastAvg;

  console.log('\n| Metric | Value |');
  console.log('| --- | --- |');
  console.log(`| Files | ${fileCount} |`);
  console.log(`| Rules | ${ruleCount} |`);
  console.log(`| Fixture Dir | ${fixtureDir} |`);
  console.log(`| rast avg | ${formatMs(rastAvg)} |`);
  console.log(`| ast-grep avg | ${formatMs(astAvg)} |`);
  console.log(`| Speedup (ast-grep / rast) | ${speedup.toFixed(2)}x |`);
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const repoRoot = path.resolve(__dirname, '..');
  const rastBinaryPath = path.resolve(repoRoot, 'packages/rast-cli/dist/index.js');
  if (!fs.existsSync(rastBinaryPath)) {
    throw new Error(
      `Rast CLI binary not found: ${rastBinaryPath}. Run "pnpm build" first.`
    );
  }

  const astGrepRunner = resolveAstGrepRunner(repoRoot);
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'rast-bench-'));
  const fixtureProjectDir = path.join(fixtureRoot, 'project');
  const fixtureRulesDir = path.join(fixtureRoot, 'rules');

  console.log('Preparing benchmark fixture...');
  createFixtureProject(fixtureProjectDir, options.files);
  const rulePaths = writeRuleFiles(
    fixtureRulesDir,
    buildRuleDefinitions().slice(0, options.rules)
  );

  try {
    console.log('Running warmup...');
    runCommand(
      rastBinaryPath,
      ['scan', fixtureProjectDir, rulePaths[0], '--dry-run', '--output', 'json'],
      repoRoot
    );
    runCommand(
      astGrepRunner.command,
      [
        ...astGrepRunner.baseArgs,
        'scan',
        fixtureProjectDir,
        '--rule',
        rulePaths[0],
        '--json=stream',
        '--color=never',
      ],
      repoRoot
    );

    console.log('Running benchmark rounds...');
    const rastTimes = benchmarkRast(
      rastBinaryPath,
      fixtureProjectDir,
      rulePaths,
      options.rounds,
      repoRoot
    );
    const astTimes = benchmarkAstGrep(
      astGrepRunner,
      fixtureProjectDir,
      rulePaths,
      options.rounds,
      repoRoot
    );

    printRoundTable(rastTimes, astTimes);
    printSummary(
      rastTimes,
      astTimes,
      options.files,
      options.rules,
      fixtureProjectDir
    );

    const rastAvg = avg(rastTimes);
    const astAvg = avg(astTimes);
    if (rastAvg >= astAvg) {
      throw new Error(
        `VERIFY-1 failed: rast avg (${formatMs(rastAvg)}) is not faster than ast-grep avg (${formatMs(astAvg)}).`
      );
    }
    console.log('\nVERIFY-1 passed: rast scan is faster than ast-grep on this benchmark.');
  } finally {
    if (!options.keepFixture) {
      fs.rmSync(fixtureRoot, { recursive: true, force: true });
    } else {
      console.log(`\nFixture preserved at: ${fixtureRoot}`);
    }
  }
}

try {
  main();
} catch (error) {
  console.error('Benchmark verification failed:', error.message);
  process.exit(1);
}
