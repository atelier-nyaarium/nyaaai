// Stdio adapter.

import { countedIds, type Ids, randomIds } from "@nyaaai/contract/id";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import {
	type AnyShape,
	CONTRACT_VERSION,
	compatibleVersions,
	decode,
	type Input,
	type Output,
	type PathPart,
} from "@nyaaai/contract/shape";
import { AdapterFrame, type FaultCode, HostFrame, type RunFailure, type Start } from "@nyaaai/contract/shapes/frames";
import { type Clock, steppedClock, systemClock } from "@nyaaai/contract/time";

////////////////////////////////
//  Interfaces & Types

export type Inbound =
	| { type: "steer"; text: string }
	| { type: "interrupt" }
	| { type: "reply"; id: string; value: unknown }
	| { type: "reply-failed"; id: string; message: string };

export interface RunContext {
	clock: Clock;
	ids: Ids;
	emit: (event: unknown) => void;
	/** Returns request ID. */
	ask: (kind: string, input: unknown) => string;
	/** Interrupts repeat forever. */
	receive: () => Promise<Inbound>;
	/** Interrupts throw while awaiting. */
	reply: (id: string) => Promise<unknown>;
	request: (kind: string, input: unknown) => Promise<unknown>;
	steers: () => string[];
	/** Aborted at an interrupt and when the input ends. */
	signal: AbortSignal;
}

export type Command = (input: unknown, context: RunContext) => Promise<unknown>;

export interface ServeOptions {
	input: AsyncIterable<Uint8Array>;
	/** Writes LF-free frames. */
	write: (line: string) => void;
	commands: ReadonlyMap<string, Command>;
}

type Awaitable<T> = T | Promise<T>;

////////////////////////////////
//  Constants

/** Oversized lines stay unread. */
export const MAX_FRAME_BYTES = 8 * 1024 * 1024;

const LF = 0x0a;

/** Rust shares this message. */
const CRASHED = "the command crashed";

export const EXIT_REJECTED = 2;

////////////////////////////////
//  Functions & Helpers

export class Interrupted extends Error {
	constructor() {
		super("interrupted");
	}
}

export class HostRequestFailed extends Error {}

export class BadInput extends Error {
	constructor(readonly path: PathPart[]) {
		super("the input does not fit the command");
	}
}

class UnknownCommand extends Error {}

/** Validates input and output. */
export function command<In extends AnyShape, Out extends AnyShape>(
	input: In,
	result: Out,
	run: (input: Output<In>, context: RunContext) => Awaitable<Input<Out>>,
): Command {
	return async (raw, context) => {
		const given = decode(input, raw);
		if (!given.ok) throw new BadInput(given.path);
		const plain = parseJson(stringifyJson(await run(given.value, context)));
		if (plain === undefined) throw new Error("the result is not JSON");
		const made = decode(result, plain);
		if (!made.ok) throw new Error("the result does not fit the command");
		return made.value;
	};
}

const INTERRUPT: Inbound = { type: "interrupt" };

const answers = (message: Inbound, id: string): boolean =>
	(message.type === "reply" || message.type === "reply-failed") && message.id === id;

/** Interrupts stop ordered delivery. */
class Inbox {
	private readonly held: Inbound[] = [];
	private waiting: ((message: Inbound) => void) | undefined;
	private readonly awaited = new Map<string, (message: Inbound) => void>();
	private interrupted = false;

	push(message: Inbound): void {
		if (this.interrupted) return;
		const waiting = this.waiting;
		if (message.type === "interrupt") {
			this.interrupted = true;
			const woken = [...this.awaited.values()];
			this.awaited.clear();
			this.waiting = undefined;
			for (const wake of woken) wake(message);
			waiting?.(message);
			return;
		}
		const awaited = message.type === "steer" ? undefined : this.awaited.get(message.id);
		if (awaited) {
			this.awaited.delete((message as { id: string }).id);
			awaited(message);
		} else if (waiting) {
			this.waiting = undefined;
			waiting(message);
		} else this.held.push(message);
	}

	next(): Promise<Inbound> {
		const first = this.held.shift();
		if (first) return Promise.resolve(first);
		if (this.interrupted) return Promise.resolve(INTERRUPT);
		return new Promise((resolve) => {
			this.waiting = resolve;
		});
	}

	reply(id: string): Promise<Inbound> {
		const at = this.held.findIndex((message) => answers(message, id));
		if (at !== -1) return Promise.resolve(this.held.splice(at, 1)[0] as Inbound);
		if (this.interrupted) return Promise.resolve(INTERRUPT);
		return new Promise((resolve) => {
			this.awaited.set(id, resolve);
		});
	}

	takeSteers(): string[] {
		const steers = this.held.flatMap((message) => (message.type === "steer" ? [message.text] : []));
		const rest = this.held.filter((message) => message.type !== "steer");
		this.held.splice(0, this.held.length, ...rest);
		return steers;
	}
}

class Run implements RunContext {
	readonly inbox = new Inbox();
	readonly pending = new Set<string>();
	private readonly abort = new AbortController();
	readonly signal = this.abort.signal;
	private seq = 0;
	private asked = 0;

	constructor(
		readonly id: string,
		readonly clock: Clock,
		readonly ids: Ids,
		private readonly write: (frame: AdapterFrame) => void,
	) {}

	nextSeq(): number {
		this.seq += 1;
		return this.seq;
	}

	emit = (event: unknown): void => {
		this.write({ type: "event", run: this.id, seq: this.nextSeq(), event: event ?? null });
	};

	ask = (kind: string, input: unknown): string => {
		this.asked += 1;
		const id = `q${this.asked}`;
		this.pending.add(id);
		this.write({ type: "request", run: this.id, id, kind, input: input ?? null });
		return id;
	};

	receive = (): Promise<Inbound> => this.inbox.next();

	reply = async (id: string): Promise<unknown> => {
		const message = await this.inbox.reply(id);
		if (message.type === "reply") return message.value;
		if (message.type === "reply-failed") throw new HostRequestFailed(message.message);
		throw new Interrupted();
	};

	request = (kind: string, input: unknown): Promise<unknown> => this.reply(this.ask(kind, input));

	steers = (): string[] => this.inbox.takeSteers();

	interrupt(): void {
		this.inbox.push(INTERRUPT);
		this.abort.abort();
	}
}

function concat(parts: Uint8Array[], size: number): Uint8Array {
	const whole = new Uint8Array(size);
	let at = 0;
	for (const part of parts) {
		whole.set(part, at);
		at += part.length;
	}
	return whole;
}

/** LF frames drop tails and signal oversize once. */
async function* frameLines(input: AsyncIterable<Uint8Array>): AsyncGenerator<Uint8Array | null> {
	let parts: Uint8Array[] = [];
	let size = 0;
	let skipping = false;
	for await (const chunk of input) {
		let from = 0;
		for (;;) {
			const at = chunk.indexOf(LF, from);
			const end = at === -1 ? chunk.length : at;
			if (!skipping) {
				size += end - from;
				if (size > MAX_FRAME_BYTES) {
					skipping = true;
					parts = [];
					yield null;
				} else if (end > from) {
					// Input buffers get reused.
					parts.push(chunk.slice(from, end));
				}
			}
			if (at === -1) break;
			if (!skipping) yield concat(parts, size);
			parts = [];
			size = 0;
			skipping = false;
			from = at + 1;
		}
	}
}

function failureOf(error: unknown): { code: RunFailure; path?: PathPart[]; message: string } {
	if (error instanceof Interrupted) return { code: "interrupted", message: error.message };
	if (error instanceof BadInput) return { code: "bad_input", path: error.path, message: error.message };
	if (error instanceof HostRequestFailed) return { code: "host_request", message: error.message };
	if (error instanceof UnknownCommand) return { code: "unknown_command", message: error.message };
	return { code: "error", message: error instanceof Error ? error.message : CRASHED };
}

export async function serve(options: ServeOptions): Promise<number> {
	const write = (frame: AdapterFrame): void => {
		const held = decode(AdapterFrame, frame);
		if (!held.ok) throw new Error(`an adapter frame at ${stringifyJson(held.path)}: ${held.message}`);
		options.write(stringifyJson(held.value));
	};
	const fault = (code: FaultCode, message: string, more: { run?: string; path?: PathPart[] } = {}): void =>
		write({ type: "fault", code, ...more, message });
	const runs = new Map<string, Run>();
	const open = new Set<Promise<void>>();
	const decoder = new TextDecoder("utf-8", { ignoreBOM: true });
	let greeted = false;

	const start = (frame: Start): void => {
		const { fixed } = frame;
		const run = new Run(
			frame.run,
			fixed ? steppedClock(fixed.nowMs, fixed.stepMs) : systemClock,
			fixed ? countedIds(fixed.idPrefix) : randomIds,
			write,
		);
		runs.set(run.id, run);
		const work = async (): Promise<AdapterFrame> => {
			try {
				const held = options.commands.get(frame.command);
				if (!held) throw new UnknownCommand(`no command ${frame.command}`);
				const result = await held(frame.input, run);
				return { type: "done", run: run.id, seq: run.nextSeq(), result: result ?? null };
			} catch (error) {
				return { type: "failed", run: run.id, seq: run.nextSeq(), ...failureOf(error) };
			}
		};
		const ended = work().then((terminal) => {
			runs.delete(run.id);
			write(terminal);
			open.delete(ended);
		});
		open.add(ended);
	};

	for await (const line of frameLines(options.input)) {
		if (line === null) {
			fault("oversized_frame", "the frame is over the size cap");
			continue;
		}
		if (line.length === 0) continue;
		const value = parseJson(decoder.decode(line));
		if (value === undefined) {
			fault("bad_frame", "not JSON");
			continue;
		}
		const decoded = decode(HostFrame, value);
		if (!decoded.ok) {
			fault("bad_frame", "not a host frame", { path: decoded.path });
			continue;
		}
		const frame = decoded.value;
		if (!greeted) {
			if (frame.type !== "hello") {
				fault("not_greeted", "say hello first");
			} else if (compatibleVersions(frame.version, CONTRACT_VERSION)) {
				greeted = true;
				write({ type: "welcome", version: CONTRACT_VERSION });
			} else {
				write({ type: "reject", version: CONTRACT_VERSION, reason: "another contract version" });
				return EXIT_REJECTED;
			}
			continue;
		}
		if (frame.type === "hello") {
			fault("bad_frame", "greeted twice");
			continue;
		}
		const run = runs.get(frame.run);
		if (frame.type === "start") {
			if (run) fault("duplicate_run", "the run is open", { run: frame.run });
			else start(frame);
		} else if (!run) {
			fault("unknown_run", "no such run", { run: frame.run });
		} else if (frame.type === "steer") {
			run.inbox.push({ type: "steer", text: frame.text });
		} else if (frame.type === "interrupt") {
			run.interrupt();
		} else if (!run.pending.delete(frame.id)) {
			fault("unknown_reply", "no such request", { run: frame.run });
		} else if (frame.type === "reply") {
			run.inbox.push({ type: "reply", id: frame.id, value: frame.value });
		} else {
			run.inbox.push({ type: "reply-failed", id: frame.id, message: frame.message });
		}
	}

	for (const run of runs.values()) run.interrupt();
	await Promise.all(open);
	return 0;
}
