// TypeScript case adapter.
// Run: bun scripts/serve.ts

import { type Command, command, HostRequestFailed, Interrupted, serve } from "@nyaaai/cli/adapter";
import { COMMANDS } from "@nyaaai/cli/commands";
import type { AnyShape } from "@nyaaai/contract/shape";
import { CASE_KINDS, TestkitConverseInput, TestkitConverseResult } from "@nyaaai/testkit/caseKinds";
import { CASE_PREFIX, HANDLERS } from "./caseAdapter";

////////////////////////////////
//  Functions & Helpers

const converse = command(TestkitConverseInput, TestkitConverseResult, async (input, context) => {
	const asked = input.together
		? Array.from({ length: input.requests }, (_, at) => context.ask("echo", { count: at + 1 }))
		: [];
	for (const settled of await Promise.allSettled(asked.map(context.reply))) {
		if (settled.status === "fulfilled") context.emit({ reply: settled.value });
		else if (settled.reason instanceof HostRequestFailed) context.emit({ failed: settled.reason.message });
		else throw settled.reason;
	}
	for (let count = 1; count <= input.requests && !input.together; count += 1) {
		const id = context.ask("echo", { count });
		for (;;) {
			const message = await context.receive();
			if (message.type === "interrupt") throw new Interrupted();
			if (message.type === "steer") {
				context.emit({ steer: message.text });
			} else if (message.id === id) {
				context.emit(message.type === "reply" ? { reply: message.value } : { failed: message.message });
				break;
			}
		}
	}
	return {
		ids: [context.ids.newId(), context.ids.newId()],
		nowMs: context.clock.nowMs(),
		steers: context.steers(),
	};
});

const caseKinds = Object.entries(CASE_KINDS).map(([kind, shapes]): [string, Command] => {
	const run = (HANDLERS as Record<string, (input: never) => unknown>)[kind] as (input: unknown) => never;
	return [`${CASE_PREFIX}${kind}`, command(shapes.input as AnyShape, shapes.result as AnyShape, run)];
});

////////////////////////////////
//  Main

if (import.meta.main) {
	const commands = new Map<string, Command>([...COMMANDS, ...caseKinds, ["testkit.converse", converse]]);
	const write = (line: string) => void process.stdout.write(`${line}\n`);
	process.exit(await serve({ input: Bun.stdin.stream(), write, commands }));
}
