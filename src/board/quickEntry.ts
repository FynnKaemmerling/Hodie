// Shorthand for new cards: "#tag" adds a tag, a standalone "!", "!!" or
// "!!!" sets the priority to low, medium or high. Everything else is the
// title. Example: "Call the dentist #health !!".
import { api, type CardSummary, type Id, type Priority, type Tag } from "../api";
import { findTagByName, nextTagColor } from "./cardState";

export interface QuickEntry {
  title: string;
  priority: Priority;
  tagNames: string[];
}

const PRIORITY_MARKS: Record<string, Priority> = { "!": "low", "!!": "medium", "!!!": "high" };

export function parseQuickEntry(text: string): QuickEntry {
  const words: string[] = [];
  const tagNames: string[] = [];
  let priority: Priority = "none";

  for (const word of text.trim().split(/\s+/)) {
    if (word in PRIORITY_MARKS) {
      priority = PRIORITY_MARKS[word];
    } else if (word.length > 1 && word.startsWith("#")) {
      const name = word.slice(1);
      if (!tagNames.some((n) => n.toLocaleLowerCase() === name.toLocaleLowerCase())) {
        tagNames.push(name);
      }
    } else if (word !== "") {
      words.push(word);
    }
  }

  return { title: words.join(" "), priority, tagNames };
}

/** Splits tag names into existing tag ids and names still to be created. */
export function resolveTags(tagNames: string[], tags: Tag[]): { ids: Id[]; missing: string[] } {
  const ids: Id[] = [];
  const missing: string[] = [];
  for (const name of tagNames) {
    const tag = findTagByName(tags, name);
    if (tag) ids.push(tag.id);
    else missing.push(name);
  }
  return { ids, missing };
}

/**
 * Creates a card from shorthand text, creating tags that do not exist yet.
 * Rejects with `validation.empty` when only markers were typed.
 */
export async function createCardFromEntry(
  columnId: Id,
  text: string,
  tags: Tag[],
): Promise<{ card: CardSummary; createdTags: Tag[] }> {
  const entry = parseQuickEntry(text);
  if (entry.title === "") throw "validation.empty";

  const { ids, missing } = resolveTags(entry.tagNames, tags);
  const createdTags: Tag[] = [];
  for (const name of missing) {
    const tag = await api.createTag(name, nextTagColor([...tags, ...createdTags]));
    createdTags.push(tag);
    ids.push(tag.id);
  }

  const card = await api.createCard({
    columnId,
    title: entry.title,
    priority: entry.priority,
    tagIds: ids,
  });
  return { card, createdTags };
}
