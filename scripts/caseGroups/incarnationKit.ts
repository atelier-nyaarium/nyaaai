import { EntryRegistry, type Incarnation } from "@nyaaai/agentic/provider/incarnation";
import type { ProviderEntry } from "@nyaaai/contract/shapes/provider";
import { scriptedProviderList } from "@nyaaai/testkit/scriptedPorts";

/** An incarnation whose entry has already left the list: what a held route of a removed entry points at. */
export function leftIncarnation(entry: ProviderEntry): Incarnation {
	const list = scriptedProviderList([entry]);
	const scratch = new EntryRegistry({ list, keys: { onChange: () => () => {} } });
	const incarnation = scratch.incarnation(entry.id);
	if (incarnation === undefined) throw new Error(`No incarnation for ${entry.id}.`);
	list.set([]);
	scratch.dispose();
	return incarnation;
}
