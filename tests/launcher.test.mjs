import assert from 'node:assert/strict';
import { chmod, copyFile, mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

const root = fileURLToPath(new URL('../', import.meta.url));

async function makeFixture(groups) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'wuwa-launcher-'));
  const tools = path.join(directory, 'tools');
  const binary = path.join(directory, 'bin', 'wuwatappo-linux');
  await mkdir(tools);
  await mkdir(path.dirname(binary));
  await copyFile(path.join(root, 'run.sh'), path.join(directory, 'run.sh'));

  const script = (name, body) => writeFile(path.join(tools, name), `#!/usr/bin/env bash\n${body}\n`, { mode: 0o755 });
  await script('id', `printf '%s\\n' '${groups}'`);
  await script('flock', 'exit 0');
  await script('getent', 'exit 0');
  await script('newgrp', 'printf "%s\\n" "$*" >> "$NEWGRP_CAPTURE"\nexec bash -c "$3"');
  await writeFile(binary, '#!/usr/bin/env bash\nprintf "launched\\n" >> "$BINARY_CAPTURE"\n', { mode: 0o755 });
  await chmod(binary, 0o755);
  return { directory, tools, binary };
}

function launch(fixture) {
  const capture = path.join(fixture.directory, 'binary.log');
  const newgrp = path.join(fixture.directory, 'newgrp.log');
  const result = spawnSync('bash', [path.join(fixture.directory, 'run.sh')], {
    encoding: 'utf8',
    env: {
      ...process.env,
      PATH: `${fixture.tools}:${process.env.PATH}`,
      XDG_RUNTIME_DIR: fixture.directory,
      BINARY_CAPTURE: capture,
      NEWGRP_CAPTURE: newgrp,
    },
  });
  assert.equal(result.status, 0, result.stderr);
  return { capture, newgrp };
}

test('launcher applies input with newgrp when the session lacks the group', async () => {
  const fixture = await makeFixture('nwj disk wheel');
  try {
    const logs = launch(fixture);
    assert.match(await readFile(logs.newgrp, 'utf8'), /^input -c /);
    assert.match(await readFile(logs.capture, 'utf8'), /launched/);
  } finally {
    await rm(fixture.directory, { recursive: true, force: true });
  }
});

test('launcher does not re-exec when the session already inherited input', async () => {
  const fixture = await makeFixture('nwj input disk wheel');
  try {
    const logs = launch(fixture);
    await assert.rejects(readFile(logs.newgrp, 'utf8'), /ENOENT/);
    assert.match(await readFile(logs.capture, 'utf8'), /launched/);
  } finally {
    await rm(fixture.directory, { recursive: true, force: true });
  }
});
