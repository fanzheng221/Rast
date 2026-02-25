import assert from 'node:assert/strict'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const bindings = require('../index.js')

const graph = bindings.initialize_graph('default')

graph.add_file('src/utils.ts', 'export function helper(): string { return "ok"; }')
graph.add_file(
  'src/app.ts',
  "import { helper } from './utils'; export function run() { return helper(); }"
)

const structureJson = graph.get_file_structure('src/app.ts')
assert.ok(structureJson)
const structure = JSON.parse(structureJson)
assert.equal(structure.language, 'tsx')

const symbolDetails = graph.get_symbol_details('run')
assert.ok(symbolDetails.length > 0)
assert.equal(JSON.parse(symbolDetails[0]).name, 'run')

const dependencies = JSON.parse(graph.analyze_dependencies(['src/app.ts']))
assert.equal(dependencies[0][0], 'src/app.ts')
assert.equal(dependencies[0][1][0].source, 'src/utils.ts')

console.log('bindings QA passed')
