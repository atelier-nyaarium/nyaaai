// Cargo crate checks.
// Run: bun scripts/crates.ts

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { ROOT } from "./child";

////////////////////////////////
//  Interfaces & Types

export interface LockedCrate {
	name: string;
	version: string;
}

export interface Exemptions {
	/** Advisory-approved young crates. */
	young: Array<{ name: string; version: string; advisory: string }>;
	/** Reviewed advisory exceptions. */
	advisories: Array<{ id: string; reason: string }>;
}

export interface CrateSources {
	/** Publish timestamp. */
	publishedAt(crate: LockedCrate): Promise<string>;
	/** Preserve order and completeness. */
	advisories(crates: LockedCrate[]): Promise<string[][]>;
	/** Unknown IDs return empty. */
	advisoryCrates(id: string): Promise<string[]>;
	now(): number;
}

////////////////////////////////
//  Constants

const REGISTRY = "registry+https://github.com/rust-lang/crates.io-index";
const MINIMUM_AGE_MS = 7 * 24 * 60 * 60 * 1000;
const AGENT = "nyaaai-gate";
const OSV_BATCH = 500;
const ADVISORY_ID = /^(RUSTSEC-\d{4}-\d{4}|GHSA(-[0-9a-z]{4}){3}|CVE-\d{4}-\d{4,})$/;

////////////////////////////////
//  Functions & Helpers

/** Path packages omit sources. */
function lockedPackages(lock: string): Array<LockedCrate & { source: string | undefined }> {
	return lock
		.split("[[package]]")
		.slice(1)
		.flatMap((held) => {
			// Stop at next table.
			const block = held.split(/^\[/m)[0] as string;
			if (/^(name|version|source) = (?!"[^"]*"$)/m.test(block))
				throw new Error("a lock entry is not as Cargo writes it");
			const field = (key: string) => new RegExp(`^${key} = "([^"]*)"$`, "m").exec(block)?.[1];
			const name = field("name");
			const version = field("version");
			return name && version ? [{ name, version, source: field("source") }] : [];
		});
}

export function lockedCrates(lock: string): LockedCrate[] {
	return lockedPackages(lock)
		.filter((held) => held.source === REGISTRY)
		.map(({ name, version }) => ({ name, version }));
}

export function foreignPackages(lock: string): string[] {
	return lockedPackages(lock)
		.filter((held) => held.source !== undefined && held.source !== REGISTRY)
		.map((held) => `${held.name}@${held.version} comes from ${held.source}`);
}

const keyOf = (crate: LockedCrate) => `${crate.name}@${crate.version}`;

function readJson<T>(file: string, fallback: T): T {
	return existsSync(file) ? (JSON.parse(readFileSync(file, "utf8")) as T) : fallback;
}

export async function checkLock(lockFile: string, sources: CrateSources): Promise<string[]> {
	const folder = path.dirname(lockFile);
	const agesFile = path.join(folder, "crate-ages.json");
	const lock = readFileSync(lockFile, "utf8");
	const crates = lockedCrates(lock);
	const ages = readJson<Record<string, string>>(agesFile, {});
	const exempt = readJson<Exemptions>(path.join(folder, "crate-exemptions.json"), { young: [], advisories: [] });
	const problems: string[] = foreignPackages(lock);

	let learned = false;
	for (const crate of crates) {
		if (ages[keyOf(crate)] !== undefined) continue;
		ages[keyOf(crate)] = await sources.publishedAt(crate);
		learned = true;
	}
	const held = Object.fromEntries(crates.map((crate) => [keyOf(crate), ages[keyOf(crate)] as string]));
	if (learned || Object.keys(ages).length !== crates.length) {
		writeFileSync(agesFile, `${JSON.stringify(held, null, "\t")}\n`);
	}

	for (const crate of crates) {
		const age = sources.now() - Date.parse(held[keyOf(crate)] as string);
		if (age >= MINIMUM_AGE_MS) continue;
		const excuse = exempt.young.find(
			(entry) => entry.name === crate.name && entry.version === crate.version && ADVISORY_ID.test(entry.advisory),
		);
		if (!excuse) problems.push(`${keyOf(crate)} was published ${Math.floor(age / 86_400_000)} days ago`);
		else if (!(await sources.advisoryCrates(excuse.advisory)).includes(crate.name))
			problems.push(`${keyOf(crate)} cites ${excuse.advisory}, which does not name it`);
	}

	const accepted = new Set(exempt.advisories.map((entry) => entry.id));
	const found = await sources.advisories(crates);
	crates.forEach((crate, at) => {
		for (const id of found[at] ?? []) {
			if (!accepted.has(id)) problems.push(`${keyOf(crate)} is named by ${id}`);
		}
	});
	return problems;
}

async function fetchJson(url: string, init?: RequestInit): Promise<unknown> {
	const response = await fetch(url, { ...init, headers: { "User-Agent": AGENT, ...init?.headers } });
	if (!response.ok) throw new Error(`${url} answered ${response.status}`);
	return response.json();
}

export const liveSources: CrateSources = {
	async publishedAt(crate) {
		const url = `https://crates.io/api/v1/crates/${crate.name}/${crate.version}`;
		const body = (await fetchJson(url)) as { version?: { created_at?: string } };
		const at = body.version?.created_at;
		if (!at) throw new Error(`${url} names no publish time`);
		return at;
	},
	async advisories(crates) {
		const found: string[][] = [];
		for (let from = 0; from < crates.length; from += OSV_BATCH) {
			const queries = crates
				.slice(from, from + OSV_BATCH)
				.map((crate) => ({ package: { ecosystem: "crates.io", name: crate.name }, version: crate.version }));
			const body = (await fetchJson("https://api.osv.dev/v1/querybatch", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ queries }),
			})) as { results?: Array<{ vulns?: Array<{ id: string }>; next_page_token?: string }> };
			if ((body.results ?? []).length !== queries.length)
				throw new Error("OSV answered for fewer crates than asked");
			for (const result of body.results ?? []) {
				if (result.next_page_token) throw new Error("OSV cut an advisory list short");
				found.push((result.vulns ?? []).map((vuln) => vuln.id));
			}
		}
		return found;
	},
	async advisoryCrates(id) {
		const url = `https://api.osv.dev/v1/vulns/${id}`;
		const response = await fetch(url, { headers: { "User-Agent": AGENT } });
		if (response.status === 404) return [];
		if (!response.ok) throw new Error(`${url} answered ${response.status}`);
		const body = (await response.json()) as {
			affected?: Array<{ package?: { ecosystem?: string; name?: string } }>;
		};
		return (body.affected ?? []).flatMap(({ package: held }) =>
			held?.ecosystem === "crates.io" && held.name ? [held.name] : [],
		);
	},
	now: () => Date.now(),
};

////////////////////////////////
//  Main

if (import.meta.main) {
	const locks = process.argv.slice(2);
	if (locks.length === 0) locks.push(path.join(ROOT, "rust", "Cargo.lock"));
	let failed = false;
	for (const lock of locks) {
		const problems = await checkLock(lock, liveSources).catch((error: unknown) => [
			error instanceof Error ? error.message : String(error),
		]);
		for (const problem of problems) console.error(`${lock}: ${problem}`);
		console.log(`crates: ${lock} ${problems.length === 0 ? "ok" : "FAILED"}`);
		failed ||= problems.length > 0;
	}
	process.exit(failed ? 1 : 0);
}
