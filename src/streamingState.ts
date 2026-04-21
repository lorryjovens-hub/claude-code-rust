// Global tracking of which conversations are currently streaming
// Used by Sidebar to show animation on active conversations

const streamingSet = new Set<string>();

export function addStreaming(conversationId: string) {
  streamingSet.add(conversationId);
  window.dispatchEvent(new CustomEvent('streaming-change'));
}

export function removeStreaming(conversationId: string) {
  streamingSet.delete(conversationId);
  window.dispatchEvent(new CustomEvent('streaming-change'));
}

export function isStreaming(conversationId: string): boolean {
  return streamingSet.has(conversationId);
}

export function getStreamingIds(): Set<string> {
  return streamingSet;
}
