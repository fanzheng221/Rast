import { copyFile, mkdir } from 'fs/promises';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function main() {
  const cargoDir = join(__dirname, '..');
  const workspaceRoot = join(cargoDir, '..', '..');
  const rustBinary = join(
    workspaceRoot,
    'target',
    'release',
    'rast' + (process.platform === 'win32' ? '.exe' : '')
  );
  const distDir = join(cargoDir, 'dist');

  await mkdir(distDir, { recursive: true });
  await copyFile(rustBinary, join(distDir, 'index.js'));

  console.log('Built @rast/cli');
}

main().catch(console.error);
