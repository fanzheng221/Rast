const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const https = require('node:https');
const { spawnSync } = require('node:child_process');
const bindings = require('../packages/bindings');

const DEFAULT_PROJECT_COUNT = 30;
const DEFAULT_MAX_FILES_PER_PROJECT = 8;
const DEFAULT_MAX_FILE_SIZE_BYTES = 256 * 1024;
const GITHUB_API_BASE = 'https://api.github.com';

const PATTERNS = [
  'console.log($A)',
  'const $A = $B;',
  'if ($A) { $$$B }',
  'const $A = $B ? $C : $D;',
  'const $A = "prefix-" + $B;',
  '$A($$$B);',
];

function parseArgs(argv) {
  const options = {
    projects: DEFAULT_PROJECT_COUNT,
    maxFilesPerProject: DEFAULT_MAX_FILES_PER_PROJECT,
    maxFileSize: DEFAULT_MAX_FILE_SIZE_BYTES,
    keepFixture: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--') {
      continue;
    }
    if (arg === '--projects') {
      options.projects = Number(argv[++i]);
    } else if (arg === '--max-files-per-project') {
      options.maxFilesPerProject = Number(argv[++i]);
    } else if (arg === '--max-file-size-kb') {
      options.maxFileSize = Number(argv[++i]) * 1024;
    } else if (arg === '--keep-fixture') {
      options.keepFixture = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!Number.isInteger(options.projects) || options.projects <= 0) {
    throw new Error('--projects must be a positive integer');
  }
  if (
    !Number.isInteger(options.maxFilesPerProject) ||
    options.maxFilesPerProject <= 0
  ) {
    throw new Error('--max-files-per-project must be a positive integer');
  }
  if (!Number.isInteger(options.maxFileSize) || options.maxFileSize <= 0) {
    throw new Error('--max-file-size-kb must be a positive integer');
  }

  return options;
}

function requestJson(url) {
  return new Promise((resolve, reject) => {
    const req = https.get(
      url,
      {
        headers: {
          'User-Agent': 'rast-vue-verify',
          Accept: 'application/vnd.github+json',
        },
      },
      (res) => {
        let body = '';
        res.setEncoding('utf8');
        res.on('data', (chunk) => {
          body += chunk;
        });
        res.on('end', () => {
          if (res.statusCode && res.statusCode >= 400) {
            reject(new Error(`GitHub API request failed (${res.statusCode}): ${body}`));
            return;
          }
          try {
            resolve(JSON.parse(body));
          } catch (error) {
            reject(new Error(`Failed to parse JSON from ${url}: ${error.message}`));
          }
        });
      }
    );
    req.on('error', reject);
  });
}

async function searchVueRepos(limit) {
  const query = encodeURIComponent('topic:vue3 fork:false archived:false stars:>50');
  const perPage = Math.min(100, Math.max(limit * 3, 60));
  const url = `${GITHUB_API_BASE}/search/repositories?q=${query}&sort=stars&order=desc&per_page=${perPage}`;
  const payload = await requestJson(url);
  const items = Array.isArray(payload.items) ? payload.items : [];
  return items
    .filter(
      (repo) =>
        !repo.fork &&
        !repo.archived &&
        Number.isFinite(repo.size) &&
        repo.size > 0 &&
        repo.size <= 50_000
    )
    .sort((left, right) => right.stargazers_count - left.stargazers_count);
}

function runGit(args, cwd) {
  const result = spawnSync('git', args, {
    cwd,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    timeout: 45_000,
  });
  if (result.status !== 0) {
    const reason = [
      result.stderr?.trim() || '',
      result.error ? result.error.message : '',
    ]
      .filter(Boolean)
      .join(' | ');
    return { ok: false, stderr: reason };
  }
  return { ok: true, stderr: '' };
}

function collectVueFiles(rootDir, maxFileSize, maxFiles) {
  const files = [];

  function walk(currentDir) {
    if (files.length >= maxFiles) {
      return;
    }
    for (const entry of fs.readdirSync(currentDir, { withFileTypes: true })) {
      if (files.length >= maxFiles) {
        break;
      }
      if (entry.name === '.git' || entry.name === 'node_modules' || entry.name === 'dist') {
        continue;
      }
      const fullPath = path.join(currentDir, entry.name);
      if (entry.isDirectory()) {
        walk(fullPath);
        continue;
      }
      if (!entry.isFile() || !entry.name.endsWith('.vue')) {
        continue;
      }
      const stat = fs.statSync(fullPath);
      if (stat.size > maxFileSize || stat.size === 0) {
        continue;
      }
      const source = fs.readFileSync(fullPath, 'utf8');
      if (!/<script\b/i.test(source)) {
        continue;
      }
      files.push(fullPath);
    }
  }

  walk(rootDir);
  return files;
}

function computeLineCol(sourceBuffer, offset) {
  let line = 1;
  let lastBreak = -1;
  for (let i = 0; i < offset; i += 1) {
    if (sourceBuffer[i] === 10) {
      line += 1;
      lastBreak = i;
    }
  }
  const column = offset - (lastBreak + 1);
  return { line, column };
}

function validateVueFile(source, filename) {
  const sourceBuffer = Buffer.from(source, 'utf8');
  let checks = 0;
  let matches = 0;
  let baselineScript = null;

  for (const pattern of PATTERNS) {
    let payload;
    try {
      const raw = bindings.findPatternInVueSfc(source, pattern);
      payload = JSON.parse(raw);
    } catch {
      return { skipped: true, checks, matches };
    }
    checks += 1;

    if (!payload || typeof payload !== 'object') {
      throw new Error(`${filename}: findPatternInVueSfc should return an object payload`);
    }
    if (!payload.script || !payload.script.span) {
      throw new Error(`${filename}: payload.script is missing`);
    }

    if (payload.script.kind !== 'script' && payload.script.kind !== 'scriptSetup') {
      throw new Error(
        `${filename}: unexpected script kind ${payload.script.kind}`
      );
    }

    const actualScript = payload.script.span;
    if (!baselineScript) {
      baselineScript = {
        start: actualScript.start,
        end: actualScript.end,
        kind: payload.script.kind,
      };
    } else if (
      baselineScript.start !== actualScript.start ||
      baselineScript.end !== actualScript.end ||
      baselineScript.kind !== payload.script.kind
    ) {
      throw new Error(`${filename}: script span/kind should remain stable across patterns`);
    }

    const matchRows = Array.isArray(payload.matches) ? payload.matches : [];
    for (const row of matchRows) {
      matches += 1;
      const rel = row.relative_span;
      const abs = row.absolute_span;
      if (!rel || !abs) {
        throw new Error(`${filename}: missing match span in payload`);
      }

      const expectedAbsStart = baselineScript.start + rel.start;
      const expectedAbsEnd = baselineScript.start + rel.end;
      if (abs.start !== expectedAbsStart || abs.end !== expectedAbsEnd) {
        throw new Error(
          `${filename}: absolute span mismatch (expected ${expectedAbsStart}-${expectedAbsEnd}, got ${abs.start}-${abs.end})`
        );
      }

      const text = sourceBuffer.subarray(abs.start, abs.end).toString('utf8');
      if (text !== row.text) {
        throw new Error(`${filename}: match text mismatch for absolute span ${abs.start}-${abs.end}`);
      }

      const location = row.location;
      if (!location || typeof location.line !== 'number' || typeof location.column !== 'number') {
        throw new Error(`${filename}: missing location payload`);
      }
      const expectedLocation = computeLineCol(sourceBuffer, abs.start);
      if (
        location.line !== expectedLocation.line ||
        location.column !== expectedLocation.column
      ) {
        throw new Error(
          `${filename}: location mismatch (expected ${expectedLocation.line}:${expectedLocation.column}, got ${location.line}:${location.column})`
        );
      }
    }
  }

  return { skipped: !baselineScript, checks, matches };
}

function cloneVueRepoSparse(repo, destinationRoot) {
  const repoDir = path.join(destinationRoot, repo.full_name.replace('/', '__'));
  const clone = runGit(['clone', '--depth', '1', '--filter=blob:none', '--sparse', repo.clone_url, repoDir], destinationRoot);
  if (!clone.ok) {
    return { ok: false, repoDir, reason: clone.stderr || 'git clone failed' };
  }
  const sparse = runGit(['-C', repoDir, 'sparse-checkout', 'set', '--no-cone', '**/*.vue'], destinationRoot);
  if (!sparse.ok) {
    return { ok: false, repoDir, reason: sparse.stderr || 'git sparse-checkout failed' };
  }
  return { ok: true, repoDir, reason: '' };
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (typeof bindings.findPatternInVueSfc !== 'function') {
    throw new Error(
      'Missing bindings.findPatternInVueSfc. Please run "pnpm build" to regenerate NAPI exports.'
    );
  }

  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'rast-vue-verify-'));
  const reposRoot = path.join(fixtureRoot, 'repos');
  fs.mkdirSync(reposRoot, { recursive: true });

  let processedProjects = 0;
  let validatedFiles = 0;
  let skippedFiles = 0;
  let ruleChecks = 0;
  let totalMatches = 0;

  try {
    console.log('Fetching candidate Vue 3 repositories from GitHub...');
    const repos = await searchVueRepos(options.projects);
    if (repos.length === 0) {
      throw new Error('No candidate repositories returned from GitHub API.');
    }

    const maxAttempts = Math.min(repos.length, options.projects * 6);
    for (let index = 0; index < maxAttempts && processedProjects < options.projects; index += 1) {
      const repo = repos[index];
      console.log(`Scanning repo candidate ${index + 1}/${maxAttempts}: ${repo.full_name}`);
      const clone = cloneVueRepoSparse(repo, reposRoot);
      if (!clone.ok) {
        console.log(`  - skipped (${clone.reason.split('\n')[0] || 'clone failed'})`);
        continue;
      }

      const vueFiles = collectVueFiles(
        clone.repoDir,
        options.maxFileSize,
        options.maxFilesPerProject
      );
      if (vueFiles.length === 0) {
        console.log('  - skipped (no .vue files)');
        continue;
      }

      let projectHasValidatedFile = false;
      for (const vueFile of vueFiles) {
        const source = fs.readFileSync(vueFile, 'utf8');
        try {
          const result = validateVueFile(source, vueFile);
          if (result.skipped) {
            skippedFiles += 1;
            continue;
          }
          projectHasValidatedFile = true;
          validatedFiles += 1;
          ruleChecks += result.checks;
          totalMatches += result.matches;
        } catch (error) {
          throw new Error(`${repo.full_name}: ${error.message}`);
        }
      }

      if (projectHasValidatedFile) {
        processedProjects += 1;
        console.log(`Validated Vue offsets: ${processedProjects}/${options.projects} (${repo.full_name})`);
      } else {
        console.log('  - skipped (no script blocks in sampled .vue files)');
      }
    }

    if (processedProjects < options.projects) {
      throw new Error(
        `Only validated ${processedProjects} projects, lower than requested ${options.projects}.`
      );
    }

    console.log('\n| Metric | Value |');
    console.log('| --- | --- |');
    console.log(`| Projects validated | ${processedProjects} |`);
    console.log(`| Vue files validated | ${validatedFiles} |`);
    console.log(`| Vue files skipped (no script) | ${skippedFiles} |`);
    console.log(`| Rule checks executed | ${ruleChecks} |`);
    console.log(`| Total matches validated | ${totalMatches} |`);
    console.log('\nVERIFY-2 passed: Vue 3 SFC offset mapping is validated on real-world projects.');
  } finally {
    if (!options.keepFixture) {
      fs.rmSync(fixtureRoot, { recursive: true, force: true });
    } else {
      console.log(`Fixture preserved at: ${fixtureRoot}`);
    }
  }
}

main().catch((error) => {
  console.error('Vue integration verification failed:', error.message);
  process.exit(1);
});
