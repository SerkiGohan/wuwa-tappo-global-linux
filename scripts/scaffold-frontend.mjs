// Creates a placeholder frontend so `cargo check`/`cargo test` work in a fresh
// clone. The real official assets are extracted from the Windows installer with
// `npm run extract` and prepared with `npm run prepare:frontend`.
import { access, copyFile, mkdir, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = fileURLToPath(new URL('../', import.meta.url));
const output = path.join(root, 'build/frontend');
const iconOutput = path.join(root, 'src-tauri/icons');
const exists = async target => access(target).then(() => true, () => false);

await mkdir(output, { recursive: true });
if (!(await exists(path.join(output, 'index.html')))) await writeFile(path.join(output, 'index.html'), `<!doctype html>
<html lang="zh-CN">
  <head><meta charset="utf-8"><title>Wuwa Tappo CN Linux</title></head>
  <body><p>Official frontend assets are not extracted.</p></body>
</html>
`);
await mkdir(iconOutput, { recursive: true });
for (const name of ['32x32.png', '128x128.png', '128x128@2x.png', 'icon.png']) {
  const target = path.join(iconOutput, name);
  if (!(await exists(target))) await copyFile(path.join(root, 'assets/placeholder-icons', name), target);
}
console.log(`Ensured placeholder frontend at ${output} and icons at ${iconOutput}`);
