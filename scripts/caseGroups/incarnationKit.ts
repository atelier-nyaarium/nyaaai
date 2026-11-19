import { type EntryRegistry, Incarnation, identityOf } from "@nyaaai/agentic/provider/incarnation";
import type { ProviderEntry } from "@nyaaai/contract/shapes/provider";

/** An incarnation whose entry left the list: what a held route of a removed entry points at. */
export function leftIncarnation(entries: EntryRegistry, entry: ProviderEntry): Incarnation {
	const incarnation = new Incarnation(0, entry.id, identityOf(entry), entry, 0, entries);
	incarnation.live = false;
	incarnation.era.retired = true;
	return incarnation;
}
