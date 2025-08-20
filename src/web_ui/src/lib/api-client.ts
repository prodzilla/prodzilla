import { API_BASE_URL } from './config';

export async function fetchProbes() {
  const response = await fetch(`${API_BASE_URL}/probes`);
  if (!response.ok) throw new Error('Failed to fetch probes');
  return response.json();
}

export async function fetchStories() {
  const response = await fetch(`${API_BASE_URL}/stories`);
  if (!response.ok) throw new Error('Failed to fetch stories');
  return response.json();
}