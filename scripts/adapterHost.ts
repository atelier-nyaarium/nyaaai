// Stdio adapter host.

import { spawnSync } from "node:child_process";
import path from "node:path";
import { CONTRACT_VERSION } from "@nyaaai/contract/shape";
import type { Project } from "@nyaaai/testkit/project";
import { cargoBin, ROOT } from "./child";
import { PROJECT_ROOT } from "./project";

////////////////////////////////
//  Interfaces & Types

export type Frame = Record<string, unknown>;

export interface AdapterCommand {
	name: string;
	argv: string[];
}

////////////////////////////////
//  Constants

const AWAIT_MS = 30_000;

export const TS_ADAPTER: AdapterCommand = { name: "TypeScript", argv: ["bun", path.join(ROOT, "scripts", "serve.ts")] };

////////////////////////////////
//  Functions & Helpers

/** Builds the project's Rust adapter. `cargo` resolves as the gate's does. */
export function rustAdapter(project: Project): AdapterCommand {
	const [program, ...args] = project.rustAdapter.build as [string, ...string[]];
	const built = spawnSync(program === "cargo" ? cargoBin() : program, args, {
		cwd: PROJECT_ROOT,
		stdio: ["ignore", "inherit", "inherit"],
	});
	if (built.status !== 0) throw new Error("the Rust adapter did not build");
	const [binary, ...rest] = project.rustAdapter.run as [string, ...string[]];
	return { name: "Rust", argv: [path.join(PROJECT_ROOT, binary), ...rest] };
}

const matches = (frame: Frame, wanted: Frame): boolean =>
	Object.entries(wanted).every(([key, value]) => JSON.stringify(frame[key]) === JSON.stringify(value));

export class AdapterHost {
	readonly frames: Frame[] = [];
	private readonly child: Bun.Subprocess<"pipe", "pipe", "inherit">;
	private readonly reading: Promise<void>;
	private readonly consumed = new Set<number>();
	private waiting: (() => void) | undefined;
	private ended = false;

	constructor(
		command: AdapterCommand,
		private readonly rewrite: (line: string) => string = (line) => line,
		private readonly awaitMs = AWAIT_MS,
	) {
		this.child = Bun.spawn(command.argv, { cwd: PROJECT_ROOT, stdin: "pipe", stdout: "pipe", stderr: "inherit" });
		this.reading = this.read();
	}

	private async read(): Promise<void> {
		const decoder = new TextDecoder();
		let held = "";
		for await (const chunk of this.child.stdout) {
			held += decoder.decode(chunk, { stream: true });
			for (let at = held.indexOf("\n"); at !== -1; at = held.indexOf("\n")) {
				this.frames.push(JSON.parse(this.rewrite(held.slice(0, at))) as Frame);
				held = held.slice(at + 1);
			}
			this.wake();
		}
		this.ended = true;
		this.wake();
	}

	private wake(): void {
		const waiting = this.waiting;
		this.waiting = undefined;
		waiting?.();
	}

	sendBytes(bytes: Uint8Array): void {
		this.child.stdin.write(bytes);
		this.child.stdin.write("\n");
		this.child.stdin.flush();
	}

	/** Sends an unterminated tail. */
	sendTail(text: string): void {
		this.child.stdin.write(text);
		this.child.stdin.flush();
	}

	sendText(line: string): void {
		this.sendBytes(new TextEncoder().encode(line));
	}

	send(frame: unknown): void {
		this.sendText(JSON.stringify(frame));
	}

	hello(): void {
		this.send({ type: "hello", version: CONTRACT_VERSION });
	}

	/** Fails on adapter exit. */
	async awaitFrame(...wanted: Frame[]): Promise<Frame> {
		const deadline = Date.now() + this.awaitMs;
		for (;;) {
			const at = this.frames.findIndex(
				(frame, index) => !this.consumed.has(index) && wanted.some((held) => matches(frame, held)),
			);
			if (at !== -1) {
				this.consumed.add(at);
				return this.frames[at] as Frame;
			}
			if (this.ended) throw new Error(`the adapter ended before ${JSON.stringify(wanted)}`);
			const left = deadline - Date.now();
			if (left <= 0) throw new Error(`no frame like ${JSON.stringify(wanted)} within ${this.awaitMs} ms`);
			await new Promise<void>((resolve) => {
				const timer = setTimeout(resolve, left);
				this.waiting = () => {
					clearTimeout(timer);
					resolve();
				};
			});
		}
	}

	async close(): Promise<number> {
		await this.child.stdin.end();
		await this.reading;
		return await this.child.exited;
	}

	kill(): void {
		this.child.kill();
	}
}

/** Serial commands per adapter. */
export async function runCommands(
	adapter: AdapterCommand,
	starts: Array<{ run: string; command: string; input: unknown }>,
	awaitMs?: number,
): Promise<Map<string, { result: unknown } | { failed: Frame }>> {
	const host = new AdapterHost(adapter, undefined, awaitMs);
	const held = new Map<string, { result: unknown } | { failed: Frame }>();
	try {
		host.hello();
		await host.awaitFrame({ type: "welcome" });
		for (const start of starts) {
			host.send({ type: "start", ...start });
			const frame = await host.awaitFrame({ type: "done", run: start.run }, { type: "failed", run: start.run });
			held.set(start.run, frame["type"] === "done" ? { result: frame["result"] } : { failed: frame });
		}
		const exit = await host.close();
		if (exit !== 0) throw new Error(`the ${adapter.name} adapter exited with ${exit}`);
	} catch (error) {
		host.kill();
		throw error;
	}
	return held;
}
