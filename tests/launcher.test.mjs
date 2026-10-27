import assert from 'node:assert/strict';
import { access, copyFile, mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { spawn, spawnSync } from 'node:child_process';
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
  await script('getent', 'exit 0');
  await script('newgrp', 'printf "%s\\n" "$*" >> "$NEWGRP_CAPTURE"\nexec bash -c "$3"');
  await writeFile(binary, '#!/usr/bin/env bash\nif [ -e /proc/self/fd/9 ]; then printf "fd9 leaked\\n" >> "$BINARY_CAPTURE"; exit 9; fi\nprintf "started\\n" >> "$BINARY_CAPTURE"\nsleep "${BINARY_SLEEP:-0}"\nif [ "${BINARY_EXIT:-0}" != 0 ]; then exit "$BINARY_EXIT"; fi\nprintf "launched\\n" >> "$BINARY_CAPTURE"\n', { mode: 0o755 });
  return { directory, tools };
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

async function waitFor(file) {
  for (let attempt = 0; attempt < 200; attempt += 1) {
    try {
      await access(file);
      return;
    } catch {
      await new Promise(resolve => setTimeout(resolve, 10));
    }
  }
  throw new Error(`Timed out waiting for ${file}`);
}

test('lock is held while the app runs and released after it exits', async () => {
  const fixture = await makeFixture('nwj input disk wheel');
  const capture = path.join(fixture.directory, 'binary.log');
  const env = {
    ...process.env,
    PATH: `${fixture.tools}:${process.env.PATH}`,
    XDG_RUNTIME_DIR: fixture.directory,
    BINARY_CAPTURE: capture,
    NEWGRP_CAPTURE: path.join(fixture.directory, 'newgrp.log'),
  };
  try {
    const first = spawn('bash', [path.join(fixture.directory, 'run.sh')], {
      env: { ...env, BINARY_SLEEP: '1' },
      stdio: 'ignore',
    });
    await waitFor(capture);

    const second = spawnSync('bash', [path.join(fixture.directory, 'run.sh')], { encoding: 'utf8', env });
    assert.equal(second.status, 0, second.stderr);
    assert.match(second.stdout, /already running/);

    await new Promise((resolve, reject) => {
      first.once('exit', code => code === 0 ? resolve() : reject(new Error(`first launcher exited with ${code}`)));
    });

    const third = spawnSync('bash', [path.join(fixture.directory, 'run.sh')], { encoding: 'utf8', env });
    assert.equal(third.status, 0, third.stderr);
    assert.match(await readFile(capture, 'utf8'), /launched/);
  } finally {
    await rm(fixture.directory, { recursive: true, force: true });
  }
});


test('launcher does not respawn the app after a non-zero exit', async () => {
  const fixture = await makeFixture('nwj disk wheel');
  const capture = path.join(fixture.directory, 'binary.log');
  try {
    const result = spawnSync('bash', [path.join(fixture.directory, 'run.sh')], {
      encoding: 'utf8',
      env: {
        ...process.env,
        PATH: `${fixture.tools}:${process.env.PATH}`,
        XDG_RUNTIME_DIR: fixture.directory,
        BINARY_CAPTURE: capture,
        NEWGRP_CAPTURE: path.join(fixture.directory, 'newgrp.log'),
        BINARY_EXIT: '42',
      },
    });
    assert.equal(result.status, 42, result.stderr);
    const log = await readFile(capture, 'utf8');
    assert.equal(log.match(/started/g)?.length, 1);
    assert.doesNotMatch(log, /launched/);
  } finally {
    await rm(fixture.directory, { recursive: true, force: true });
  }
});
