// The adapter on this process's standard streams.

import { type Command, serve } from "./runtime";

////////////////////////////////
//  Functions & Helpers

/** Serves until end of input; resolves with the exit code. */
export async function serveStdio(commands: ReadonlyMap<string, Command>): Promise<number> {
	const sink = Bun.stdout.writer();
	let gone = false;
	const write = (line: string): void => {
		if (gone) return;
		try {
			sink.write(`${line}\n`);
			Promise.resolve(sink.flush()).catch(() => {
				gone = true;
			});
		} catch {
			gone = true;
		}
	};
	const code = await serve({ input: Bun.stdin.stream(), write, commands });
	// An exit right after a write drops what the pipe has not taken.
	await Promise.resolve(sink.end()).catch(() => undefined);
	return code;
}
