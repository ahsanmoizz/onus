const API_BASE = '/api';

export class OnusApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'OnusApiError';
  }
}

async function fetchApi<T>(path: string, options?: RequestInit): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options?.headers as Record<string, string>),
  };

  // Add CSRF token for state-changing methods
  const method = options?.method || 'GET';
  if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(method)) {
    headers['X-CSRF-Token'] = '1';
  }

  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers,
  });
  if (!res.ok) {
    throw new OnusApiError(res.status, await res.text());
  }
  return res.json();
}

export const api = {
  get: <T>(path: string) => fetchApi<T>(path),
  post: <T>(path: string, body?: unknown) =>
    fetchApi<T>(path, { method: 'POST', body: body ? JSON.stringify(body) : undefined }),
  put: <T>(path: string, body?: unknown) =>
    fetchApi<T>(path, { method: 'PUT', body: body ? JSON.stringify(body) : undefined }),
  delete: <T>(path: string) => fetchApi<T>(path, { method: 'DELETE' }),
};

export interface SystemStatus {
  daemon: 'RUNNING' | 'STOPPED';
  daemon_pid?: number;
  version: string;
  guardian_mode?: string;
  provider_mode?: string;
  total_actions?: number;
  blocked_actions?: number;
  escalated_actions?: number;
}

export async function getStatus(): Promise<SystemStatus> {
  return api.get('/status');
}

export interface Session {
  id: string;
  agent_name: string;
  task_description: string;
  workspace_root: string;
  status: string;
  total_actions: number;
  blocked_actions: number;
  escalated_actions: number;
  started_at: string;
  ended_at?: string;
}

export async function getSessions(): Promise<Session[]> {
  return api.get('/sessions');
}

export async function getSession(id: string): Promise<Session> {
  return api.get(`/sessions/${id}`);
}

export interface Action {
  id: string;
  session_id: string;
  sequence: number;
  action_type: string;
  tool_name?: string;
  verdict: string;
  rule_id?: string;
  correction?: string;
  created_at: string;
}

export async function getRecentActions(limit = 20): Promise<Action[]> {
  return api.get(`/actions?limit=${limit}`);
}

export interface Approval {
  id: string;
  session_id: string;
  action_id: string;
  payload_hash: string;
  task_contract_hash: string;
  policy_version: string;
  status: string;
  expires_at: number;
  created_at: string;
}

export async function getApprovals(status = 'pending'): Promise<Approval[]> {
  return api.get(`/approvals?status=${status}`);
}

export async function approveApproval(id: string): Promise<void> {
  return api.post(`/approvals/${id}/approve`);
}

export async function denyApproval(id: string, reason?: string): Promise<void> {
  return api.post(`/approvals/${id}/deny`, { reason });
}

export interface VerifyResult {
  status: string;
  broken_links: number;
  message: string;
}

export async function verifyChain(sessionId?: string): Promise<VerifyResult> {
  const params = sessionId ? `?session_id=${sessionId}` : '';
  return api.get(`/verify${params}`);
}

export interface DoctorResult {
  checks: Array<{
    name: string;
    status: 'ok' | 'warn' | 'fail';
    message: string;
  }>;
}

export async function runDoctor(): Promise<DoctorResult> {
  return api.get('/doctor');
}

export interface Checkpoint {
  id: string;
  session_id: string;
  description: string;
  created_at: string;
}

export async function getCheckpoints(): Promise<Checkpoint[]> {
  return api.get('/checkpoints');
}

export async function createCheckpoint(session: string, description?: string): Promise<Checkpoint> {
  return api.post('/checkpoints', { session, description });
}
