/**
 * Preprocesses piped [[Node Title|node_id]] format and standard [[Node Title]] format
 * into their corresponding markdown link representations.
 */
export function preprocessWikiLinks(text: string): string {
  if (!text) return "";
  // 1. Process piped [[Title|id]] format -> [Title](#node/id)
  let processed = text.replace(/\[\[([^|\]\n]+)\|([^\]\n]+)\]\]/g, "[$1](#node/$2)");
  // 2. Process standard [[Title]] format -> [Title](#node/search:Title)
  processed = processed.replace(/\[\[([^|\]\n]+)\]\]/g, "[$1](#node/search:$1)");
  return processed;
}
