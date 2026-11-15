// Scripted-run commands.

import { type Command, command, HostRequestFailed, Interrupted } from "@nyaaai/adapter/runtime";
import { TestkitConverseInput, TestkitConverseResult } from "./caseGroups/testkit";

/** Ends with no message: a thrown value here, a panic in Rust. */
export const crash: Command = async () => {
	throw "crash";
};

/** Reports host replies. */
export const converse = command(TestkitConverseInput, TestkitConverseResult, async (input, context) => {
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
