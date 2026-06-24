import type { Node } from "../ipc";

export function parsePriorityJson(priority: string): Record<string, unknown> {
  try {
    const parsed = JSON.parse(priority);
    if (typeof parsed === "object" && parsed !== null) {
      return parsed as Record<string, unknown>;
    }
  } catch {
    // ignore
  }
  return {};
}

export function getPriorityScore(node: Node): number {
  const obj = parsePriorityJson(node.priority);
  if (typeof obj.score === "number" && Number.isFinite(obj.score)) {
    return obj.score;
  }
  return 1.0;
}

export function getPriorityProfile(node: Node): string {
  const obj = parsePriorityJson(node.priority);
  if (typeof obj.profile === "string") {
    return obj.profile;
  }
  return "standard";
}

export function isFrozen(node: Node): boolean {
  const obj = parsePriorityJson(node.priority);
  return obj.frozen === true;
}

export function getAccessCount(node: Node, key: string): number {
  const obj = parsePriorityJson(node.priority);
  const val = obj[key];
  if (typeof val === "number" && Number.isFinite(val)) {
    return val;
  }
  return 0;
}
