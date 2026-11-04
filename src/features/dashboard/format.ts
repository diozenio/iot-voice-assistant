export function formatTime(ts: string): string {
  return new Date(ts).toLocaleTimeString("pt-BR");
}

export function formatDateTime(ts: string): string {
  return new Date(ts).toLocaleString("pt-BR");
}

export function formatDuration(seconds: number): string {
  return `${seconds.toFixed(1)}s`;
}
