const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const bindings = require('../packages/bindings');

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function getTextContent(result, context) {
  assert(result && Array.isArray(result.content), `${context}: missing content array`);
  assert(result.content[0] && typeof result.content[0].text === 'string', `${context}: missing text`);
  return result.content[0].text;
}

function ruleYaml() {
  return [
    'id: no-console',
    'language: ts',
    'rule:',
    '  pattern: console.log($A)',
    'fix: logger.info($A)',
    '',
  ].join('\n');
}

function createFixtureProject() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'rast-verify-napi-mcp-'));
  const srcDir = path.join(root, 'src');
  fs.mkdirSync(srcDir, { recursive: true });

  const files = [
    {
      file: path.join(srcDir, 'a.ts'),
      content: 'console.log(1);',
    },
    {
      file: path.join(srcDir, 'b.ts'),
      content: 'console.log(2);',
    },
  ];

  for (const row of files) {
    fs.writeFileSync(row.file, row.content, 'utf8');
  }

  return { root, files };
}

function resetFixture(files) {
  for (const row of files) {
    fs.writeFileSync(row.file, row.content, 'utf8');
  }
}

function assertRewritten(files) {
  for (const row of files) {
    const rewritten = fs.readFileSync(row.file, 'utf8');
    assert(
      rewritten.includes('logger.info('),
      `Expected logger.info codemod in ${row.file}`
    );
    assert(!rewritten.includes('console.log('), `Expected console.log removed in ${row.file}`);
  }
}

function runNapiCodemod(root) {
  const resultText = bindings.scan_directory(root, ruleYaml(), false);
  const rows = JSON.parse(resultText);
  assert(Array.isArray(rows), 'NAPI scan_directory should return array');
  const totalMatches = rows.reduce((sum, row) => sum + (row.matches || 0), 0);
  assert(totalMatches >= 2, 'NAPI scan_directory should rewrite all fixture files');
}

async function runMcpCodemod(root) {
  const { Client } = await import('@modelcontextprotocol/sdk/client/index.js');
  const { StdioClientTransport } = await import('@modelcontextprotocol/sdk/client/stdio.js');
  const mcpPath = path.resolve(__dirname, '../packages/mcp-server/dist/index.js');

  const transport = new StdioClientTransport({
    command: process.execPath,
    args: [mcpPath],
  });
  const client = new Client({ name: 'rast-verify-client', version: '0.1.0' }, { capabilities: {} });

  await client.connect(transport);
  try {
    const listed = await client.listTools();
    const toolNames = new Set(listed.tools.map((tool) => tool.name));
    assert(toolNames.has('scan_directory'), 'MCP tool list missing scan_directory');

    const result = await client.callTool({
      name: 'scan_directory',
      arguments: {
        rootPath: root,
        rule: ruleYaml(),
        dryRun: false,
      },
    });
    const text = getTextContent(result, 'MCP scan_directory');
    const rows = JSON.parse(text);
    assert(Array.isArray(rows), 'MCP scan_directory should return array JSON');
    const totalMatches = rows.reduce((sum, row) => sum + (row.matches || 0), 0);
    assert(totalMatches >= 2, 'MCP scan_directory should rewrite all fixture files');
  } finally {
    await client.close();
  }
}

async function main() {
  const fixture = createFixtureProject();
  try {
    console.log('Phase 1/2: Running codemod through NAPI bindings...');
    runNapiCodemod(fixture.root);
    assertRewritten(fixture.files);

    console.log('Phase 2/2: Running codemod through MCP (stdio, AI-style tool call)...');
    resetFixture(fixture.files);
    await runMcpCodemod(fixture.root);
    assertRewritten(fixture.files);

    console.log('\n| Check | Result |');
    console.log('| --- | --- |');
    console.log('| NAPI codemod | PASS |');
    console.log('| MCP codemod (stdio) | PASS |');
    console.log('\nVERIFY-3 passed: NAPI + MCP Codemod E2E is fully validated.');
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
}

main().catch((error) => {
  console.error('NAPI & MCP E2E verification failed:', error.message);
  process.exit(1);
});
