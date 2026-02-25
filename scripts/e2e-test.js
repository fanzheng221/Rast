const { analyzeAst } = require('../packages/bindings');
const { spawn } = require('child_process');
const path = require('path');
const { rastUnplugin } = require('../packages/unplugin/dist/index.cjs');

async function runTests() {
  try {
    // Test 1: Direct bindings call
    console.log('Test 1: Direct bindings call...');
    const result1 = analyzeAst('export const x = 1;');
    const parsed1 = JSON.parse(result1);
    if (!parsed1.exports || !Array.isArray(parsed1.exports)) {
      throw new Error('Test 1 failed: exports should be an array');
    }
    console.log('✓ Test 1 passed');

    // Test 2: With issues
    console.log('Test 2: Code with linting issues...');
    const result2 = analyzeAst('var x = 1;');
    const parsed2 = JSON.parse(result2);
    if (!parsed2.issues || !Array.isArray(parsed2.issues)) {
      throw new Error('Test 2 failed: issues should be an array');
    }
    if (parsed2.issues.length === 0) {
      throw new Error('Test 2 failed: expected at least one issue');
    }
    console.log('✓ Test 2 passed');

    // Test 3: Unplugin
    console.log('Test 3: Unplugin transform...');
    const plugin = rastUnplugin.vite({ injectIssues: true, logIssues: false });
    const transformResult = plugin.transform('var x = 1;', 'test.js');
    if (!transformResult || !transformResult.code) {
      throw new Error('Test 3 failed: transform should return code');
    }
    if (!transformResult.code.includes('Rast Issues:')) {
      throw new Error('Test 3 failed: transform should inject issues comment');
    }
    console.log('✓ Test 3 passed');

    // Test 4: MCP server
    console.log('Test 4: MCP server communication...');
    await new Promise((resolve, reject) => {
      const mcpPath = path.resolve(__dirname, '../packages/mcp-server/dist/index.js');
      const proc = spawn('node', [mcpPath]);
      
      let output = '';
      
      proc.stdout.on('data', (data) => {
        output += data.toString();
        try {
          // Try to parse the output as JSON-RPC response
          const lines = output.split('\n').filter(line => line.trim());
          for (const line of lines) {
            const response = JSON.parse(line);
            if (response.id === 3) {
              if (response.error) {
                reject(new Error(`MCP Error: ${response.error.message}`));
                return;
              }
              
              const result = response.result;
              if (!result || !result.content || !result.content[0] || !result.content[0].text) {
                reject(new Error('Invalid MCP response format'));
                return;
              }
              
              const astResult = JSON.parse(result.content[0].text);
              if (!astResult.exports) {
                reject(new Error('MCP response missing exports'));
                return;
              }
              
              console.log('✓ Test 4 passed');
              proc.kill();
              resolve();
              return;
            }
          }
        } catch (e) {
          // Ignore parse errors as we might not have received the full JSON yet
        }
      });
      
      proc.stderr.on('data', (data) => {
        console.error(`MCP stderr: ${data}`);
      });
      
      proc.on('error', (err) => {
        reject(new Error(`Failed to start MCP server: ${err.message}`));
      });
      
      proc.on('close', (code) => {
        if (code !== 0 && code !== null) {
          reject(new Error(`MCP server exited with code ${code}`));
        }
      });

      // Send request
      const request = {
        jsonrpc: '2.0',
        id: 3,
        method: 'tools/call',
        params: {
          name: 'analyze_ast',
          arguments: { source: 'export const y = 2;' }
        }
      };
      proc.stdin.write(JSON.stringify(request) + '\n');
    });

    console.log('All E2E tests passed!');
    process.exit(0);
  } catch (error) {
    console.error('E2E test failed:', error);
    process.exit(1);
  }
}

runTests();
