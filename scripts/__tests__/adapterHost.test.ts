// Adapter host tests.

import { describe, expect, it } from "bun:test";
import { existsSync } from "node:fs";
import { CONTRACT_VERSION, z } from "@nyaaai/contract/shape";
import { kind } from "@nyaaai/testkit/caseGroup";
import { rethrown } from "@nyaaai/testkit/rejection";
import { AdapterHost, TS_ADAPTER } from "../adapterHost";
import { runKind } from "../caseAdapter";

////////////////////////////////
//  Tests

describe("the adapter host", () => {
	it("fails a wait when the adapter dies, and reports its exit", async () => {
		const crashed = {
			name: "crashing",
			argv: ["bun", "-e", 'console.log(JSON.stringify({type:"welcome"})); process.exit(3)'],
		};
		const host = new AdapterHost(crashed);
		expect((await host.awaitFrame({ type: "welcome" }))["type"]).toBe("welcome");
		const waited = await host.awaitFrame({ type: "done" }).then(
			() => "came",
			() => "failed",
		);
		expect(waited).toBe("failed");
		expect(await host.close()).toBe(3);
	});

	it("delivers a large result whole when input ends at once", async () => {
		const text = JSON.stringify({ big: "x".repeat(2_000_000) });
		const host = new AdapterHost(TS_ADAPTER, undefined, 60_000);
		host.hello();
		host.send({ type: "start", run: "a", command: "case:json.canonical", input: { texts: [text] } });
		expect(await host.close()).toBe(0);
		expect(host.frames.find((frame) => frame["type"] === "done")?.["result"]).toEqual({ texts: [text] });
	});

	it("exits quietly when the reader closes after the first frame", async () => {
		const child = Bun.spawn(TS_ADAPTER.argv, { stdin: "pipe", stdout: "pipe", stderr: "pipe" });
		child.stdin.write(`${JSON.stringify({ type: "hello", version: CONTRACT_VERSION })}\n`);
		for (let n = 0; n < 300; n += 1) {
			const input = { texts: ["{}"] };
			child.stdin.write(
				`${JSON.stringify({ type: "start", run: `r${n}`, command: "case:json.canonical", input })}\n`,
			);
		}
		await child.stdin.end();
		const reader = child.stdout.getReader();
		await reader.read();
		await reader.cancel();
		expect(await child.exited).toBe(0);
		expect(await new Response(child.stderr).text()).toBe("");
	});

	it("removes a context kind's folder after a return and after a throw", async () => {
		let folder = "";
		const folders = kind(
			z.object({ fail: z.boolean() }),
			z.object({}),
			(input, context) => {
				folder = context.tmp();
				if (input.fail) throw new Error("fails");
				return {};
			},
			{ context: true },
		);
		await runKind(folders, { fail: false });
		expect(folder).not.toBe("");
		expect(existsSync(folder)).toBe(false);
		expect(await rethrown(runKind(folders, { fail: true }))).toThrow("fails");
		expect(existsSync(folder)).toBe(false);
	});
});
