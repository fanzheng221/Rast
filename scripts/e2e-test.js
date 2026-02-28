const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { analyzeAst, applyRule, scanDirectory } = require('../packages/bindings');
const { rastUnplugin } = require('../packages/unplugin/dist/index.cjs');

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function getTextContent(result, context) {
  assert(result && Array.isArray(result.content), `${context}: missing content array`);
  assert(result.content[0] && typeof result.content[0].text === 'string', `${context}: missing text payload`);
  return result.content[0].text;
}

function createTempDir(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), `rast-${prefix}-`));
}

function cleanupDir(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

function noConsoleRuleYaml() {
  return [
    'id: no-console',
    'language: ts',
    'rule:',
    '  pattern: console.log($A)',
    'fix: logger.info($A)',
    '',
  ].join('\n');
}

async function runBindingsChecks() {
  console.log('Test 1: Direct bindings call...');
  const analysis = JSON.parse(analyzeAst('export const x = 1;'));
  assert(Array.isArray(analysis.exports), 'Test 1 failed: exports should be an array');
  console.log('✓ Test 1 passed');

  console.log('Test 2: Code with linting issues...');
  const withIssues = JSON.parse(analyzeAst('var x = 1;'));
  assert(Array.isArray(withIssues.issues), 'Test 2 failed: issues should be an array');
  assert(withIssues.issues.length > 0, 'Test 2 failed: expected at least one issue');
  console.log('✓ Test 2 passed');

  console.log('Test 3: NAPI codemod apply_rule + scan_directory...');
  const updated = applyRule('console.log(foo);', noConsoleRuleYaml());
  assert(updated === 'logger.info(foo)', 'Test 3 failed: applyRule output mismatch');

  const scanDir = createTempDir('scan');
  const targetFile = path.join(scanDir, 'sample.ts');
  fs.writeFileSync(targetFile, 'console.log(bar);', 'utf8');
  try {
    const result = JSON.parse(scanDirectory(scanDir, noConsoleRuleYaml(), false));
    assert(Array.isArray(result), 'Test 3 failed: scanDirectory should return JSON array');
    assert(result.length === 1, 'Test 3 failed: expected one scanned file');
    assert(result[0].matches === 1, 'Test 3 failed: expected one match');
    const rewritten = fs.readFileSync(targetFile, 'utf8');
    assert(rewritten === 'logger.info(bar)', 'Test 3 failed: file should be rewritten by scanDirectory');
  } finally {
    cleanupDir(scanDir);
  }
  console.log('✓ Test 3 passed');
}

function runUnpluginChecks() {
  console.log('Test 4: Unplugin transform...');
  const plugin = rastUnplugin.vite({ injectIssues: true, logIssues: false });
  const transformResult = plugin.transform('var x = 1;', 'test.js');
  assert(transformResult && transformResult.code, 'Test 4 failed: transform should return code');
  assert(transformResult.code.includes('Rast Issues:'), 'Test 4 failed: transform should inject issues comment');
  console.log('✓ Test 4 passed');
}

async function runMcpDirectChecks() {
  console.log('Test 5: MCP direct module callTool...');
  const mcpPath = path.resolve(__dirname, '../packages/mcp-server/dist/index.js');
  const mcpModule = await import(mcpPath);
  const { callTool, tools } = mcpModule;

  assert(Array.isArray(tools), 'Test 5 failed: tools should be an array');
  const toolNames = new Set(tools.map((tool) => tool.name));
  assert(toolNames.has('findPattern'), 'Test 5 failed: findPattern tool missing');
  assert(toolNames.has('applyRule'), 'Test 5 failed: applyRule tool missing');
  assert(toolNames.has('scanDirectory'), 'Test 5 failed: scanDirectory tool missing');

  const applyResult = await callTool('applyRule', {
    source: 'console.log(value);',
    rule: noConsoleRuleYaml(),
  });
  assert(applyResult === 'logger.info(value)', 'Test 5 failed: MCP applyRule result mismatch');

  const scanDir = createTempDir('mcp-direct');
  const targetFile = path.join(scanDir, 'entry.ts');
  fs.writeFileSync(targetFile, 'console.log(data);', 'utf8');
  try {
    const scanResultText = await callTool('scanDirectory', {
      rootPath: scanDir,
      rule: noConsoleRuleYaml(),
      dryRun: false,
    });
    const scanResult = JSON.parse(scanResultText);
    assert(Array.isArray(scanResult), 'Test 5 failed: MCP scanDirectory should return JSON array');
    assert(scanResult[0].matches === 1, 'Test 5 failed: MCP scanDirectory should find one match');
    assert(fs.readFileSync(targetFile, 'utf8') === 'logger.info(data)', 'Test 5 failed: MCP scanDirectory should rewrite file');
  } finally {
    cleanupDir(scanDir);
  }
  console.log('✓ Test 5 passed');
}

async function runMcpStdioCommunicationChecks() {
  console.log('Test 6: MCP stdio communication...');
  const { Client } = await import('@modelcontextprotocol/sdk/client/index.js');
  const { StdioClientTransport } = await import('@modelcontextprotocol/sdk/client/stdio.js');
  const mcpPath = path.resolve(__dirname, '../packages/mcp-server/dist/index.js');

  const transport = new StdioClientTransport({
    command: process.execPath,
    args: [mcpPath],
  });
  const client = new Client({ name: 'rast-e2e-client', version: '0.1.0' }, { capabilities: {} });

  await client.connect(transport);
  const scanDir = createTempDir('mcp-stdio');
  const targetFile = path.join(scanDir, 'stdio.ts');
  fs.writeFileSync(targetFile, 'console.log(stdioValue);', 'utf8');

  try {
    const listed = await client.listTools();
    const toolNames = new Set(listed.tools.map((tool) => tool.name));
    assert(toolNames.has('applyRule'), 'Test 6 failed: stdio listTools missing applyRule');
    assert(toolNames.has('scanDirectory'), 'Test 6 failed: stdio listTools missing scanDirectory');

    const applyResult = await client.callTool({
      name: 'applyRule',
      arguments: {
        source: 'console.log(viaMcp);',
        rule: noConsoleRuleYaml(),
      },
    });
    const applyText = getTextContent(applyResult, 'Test 6 applyRule');
    assert(applyText === 'logger.info(viaMcp)', 'Test 6 failed: stdio applyRule result mismatch');

    const scanResult = await client.callTool({
      name: 'scanDirectory',
      arguments: {
        rootPath: scanDir,
        rule: noConsoleRuleYaml(),
        dryRun: false,
      },
    });
    const scanText = getTextContent(scanResult, 'Test 6 scanDirectory');
    const parsed = JSON.parse(scanText);
    assert(Array.isArray(parsed), 'Test 6 failed: stdio scanDirectory should return JSON array');
    assert(parsed[0].matches === 1, 'Test 6 failed: stdio scanDirectory should find one match');
    assert(
      fs.readFileSync(targetFile, 'utf8') === 'logger.info(stdioValue)',
      'Test 6 failed: stdio scanDirectory should rewrite file'
    );
  } finally {
    cleanupDir(scanDir);
    await client.close();
  }
  console.log('✓ Test 6 passed');
}

async function runTests() {
  try {
    await runBindingsChecks();
    runUnpluginChecks();
    await runMcpDirectChecks();
    await runMcpStdioCommunicationChecks();
    console.log('All E2E tests passed!');
    process.exit(0);
  } catch (error) {
    console.error('E2E test failed:', error);
    process.exit(1);
  }
}

runTests();
