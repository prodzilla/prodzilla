const API_BASE_URL = 'http://localhost:3000';

export type MonitorItem = {
  name: string;
  status: string;
  last_probed: string;
  tags: Record<string, string>;
};

export type StoryResult = {
  story_name?: string;
  timestamp_started: string;
  success: boolean;
  step_results: {
    step_name: string;
    timestamp_started: string;
    success: boolean;
    error_message?: string;
    trace_id?: string;
    span_id?: string;
  }[];
};

export type ProbeResult = {
  probe_name: string;
  timestamp_started: string;
  success: boolean;
  error_message?: string;
  trace_id?: string;
};

export type BulkProbeResult = {
  triggered_count: number;
  results: ProbeResult[];
};

export type BulkStoryResult = {
  triggered_count: number;
  results: StoryResult[];
};

export async function fetchProbes() {
  const response = await fetch(`${API_BASE_URL}/probes`);
  if (!response.ok) throw new Error('Failed to fetch probes');
  return response.json() as Promise<MonitorItem[]>;
}

export async function fetchStories() {
  const response = await fetch(`${API_BASE_URL}/stories`);
  if (!response.ok) throw new Error('Failed to fetch stories');
  return response.json() as Promise<MonitorItem[]>;
}

export const isStoryResult = (
  result: ProbeResult | StoryResult
): result is StoryResult => {
  return 'story_name' in result;
};

export async function fetchProbeResults(name: string) {
  const response = await fetch(
    `${API_BASE_URL}/probes/${encodeURIComponent(name)}/results`
  );
  if (!response.ok) throw new Error('Failed to fetch probe results');
  return response.json() as Promise<ProbeResult[]>;
}

export async function fetchStoryResults(name: string) {
  const response = await fetch(
    `${API_BASE_URL}/stories/${encodeURIComponent(name)}/results`
  );
  if (!response.ok) throw new Error('Failed to fetch story results');
  return response.json() as Promise<StoryResult[]>;
}

export async function bulkTriggerProbes(tags: Record<string, string>) {
  const response = await fetch(`${API_BASE_URL}/probes/bulk/trigger`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tags }),
  });
  if (!response.ok) throw new Error('Failed to trigger probes');
  const data = await response.json();
  return data as BulkProbeResult;
}

export async function bulkTriggerStories(tags: Record<string, string>) {
  const response = await fetch(`${API_BASE_URL}/stories/bulk/trigger`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tags }),
  });
  if (!response.ok) throw new Error('Failed to trigger stories');
  const data = await response.json();
  return data as BulkStoryResult;
}
