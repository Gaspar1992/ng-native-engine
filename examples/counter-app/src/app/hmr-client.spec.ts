import { unlinkSync } from 'node:fs';
import net from 'node:net';
import { pingHmr, pushHmrEval, pushHmrReload, sendHmrMessage } from '../../hmr-client.mjs';

describe('HMR IPC Client (Node.js)', () => {
  const testSocket = `/tmp/ng-native-hmr-vitest-${process.pid}.sock`;
  let server: net.Server;

  beforeAll(async () => {
    try {
      unlinkSync(testSocket);
    } catch {
      // Ignore if socket file doesn't exist
    }

    server = net.createServer((conn: net.Socket) => {
      conn.on('data', (data: Buffer) => {
        const line = data.toString().trim();
        const msg = JSON.parse(line);
        if (msg.type === 'ping') {
          conn.write(JSON.stringify({ status: 'ok', pong: true }) + '\n');
        } else if (msg.type === 'reload') {
          conn.write(
            JSON.stringify({ status: 'ok', reloaded: true, length: msg.script.length }) + '\n',
          );
        } else if (msg.type === 'eval') {
          conn.write(JSON.stringify({ status: 'ok', result: `eval:${msg.code}` }) + '\n');
        }
      });
    });

    await new Promise<void>((resolve) => {
      server.listen(testSocket, () => resolve());
    });
  });

  afterAll(async () => {
    await new Promise<void>((resolve) => {
      server.close(() => {
        try {
          unlinkSync(testSocket);
        } catch {
          // Ignore
        }
        resolve();
      });
    });
  });

  it('pings running server and parses pong response', async () => {
    const res = await pingHmr(testSocket);
    expect(res.connected).toBe(true);
    expect(res.response).toEqual({ status: 'ok', pong: true });
  });

  it('pushes bundle reload and handles script payload', async () => {
    const res = await pushHmrReload(testSocket, 'signal count 42\ntext win App');
    expect(res.connected).toBe(true);
    expect(res.response).toEqual({ status: 'ok', reloaded: true, length: 28 });
  });

  it('evaluates JS snippets over IPC', async () => {
    const res = await pushHmrEval(testSocket, '__NG_NATIVE__.getTheme()');
    expect(res.connected).toBe(true);
    expect(res.response).toEqual({ status: 'ok', result: 'eval:__NG_NATIVE__.getTheme()' });
  });

  it('gracefully handles missing socket when engine is stopped', async () => {
    const res = await sendHmrMessage('/tmp/nonexistent-engine.sock', { type: 'ping' });
    expect(res.connected).toBe(false);
    expect(res.error).toBe('Socket file not found');
  });
});
