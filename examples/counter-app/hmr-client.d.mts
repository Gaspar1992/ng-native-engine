export interface HmrClientResult {
  connected: boolean;
  response?: Record<string, unknown>;
  rawResponse?: string;
  error?: string;
  timeout?: boolean;
}

export function sendHmrMessage(
  socketPath: string,
  message: { type: string; [key: string]: unknown },
  options?: { timeoutMs?: number },
): Promise<HmrClientResult>;

export function pushHmrReload(socketPath: string, bundleScript: string): Promise<HmrClientResult>;

export function pushHmrEval(socketPath: string, jsCode: string): Promise<HmrClientResult>;

export function pingHmr(socketPath: string): Promise<HmrClientResult>;
