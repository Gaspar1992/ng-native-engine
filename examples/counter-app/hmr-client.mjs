import { existsSync } from 'node:fs';
import net from 'node:net';

/**
 * Sends an HMR IPC message to the running native Rust engine over a local Unix domain socket.
 * Gracefully resolves with `{ connected: false }` if the engine is not currently running.
 */
export function sendHmrMessage(socketPath, message, { timeoutMs = 1500 } = {}) {
  return new Promise((resolve) => {
    if (!existsSync(socketPath)) {
      resolve({ connected: false, error: 'Socket file not found' });
      return;
    }

    const client = net.createConnection(socketPath);
    let resolved = false;
    let responseData = '';

    const timer = setTimeout(() => {
      if (!resolved) {
        resolved = true;
        client.destroy();
        resolve({ connected: true, timeout: true });
      }
    }, timeoutMs);

    client.on('connect', () => {
      client.write(JSON.stringify(message) + '\n');
    });

    client.on('data', (chunk) => {
      responseData += chunk.toString();
      if (responseData.includes('\n')) {
        const line = responseData.split('\n')[0].trim();
        clearTimeout(timer);
        if (!resolved) {
          resolved = true;
          client.end();
          try {
            resolve({ connected: true, response: JSON.parse(line) });
          } catch {
            resolve({ connected: true, rawResponse: line });
          }
        }
      }
    });

    client.on('error', (err) => {
      clearTimeout(timer);
      if (!resolved) {
        resolved = true;
        resolve({ connected: false, error: err.message });
      }
    });
  });
}

/**
 * Pushes a newly compiled `.ngb` bundle to the live running desktop window.
 */
export async function pushHmrReload(socketPath, bundleScript) {
  return sendHmrMessage(socketPath, { type: 'reload', script: bundleScript });
}

/**
 * Evaluates arbitrary JavaScript inside the active runtime of the running desktop window.
 */
export async function pushHmrEval(socketPath, jsCode) {
  return sendHmrMessage(socketPath, { type: 'eval', code: jsCode });
}

/**
 * Pings the running desktop engine to check IPC connectivity.
 */
export async function pingHmr(socketPath) {
  return sendHmrMessage(socketPath, { type: 'ping' });
}
