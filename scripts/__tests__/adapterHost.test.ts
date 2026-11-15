// Adapter host tests.

import { describe, expect, it } from "bun:test";
import { AdapterHost } from "../adapterHost";

////////////////////////////////
//  Tests

describe("the adapter host", () => {
	it("fails a wait when the adapter dies, and reports its exit", async () => {
		const crashed = {
			name: "crashing",
			argv: ["bun", "-e", 'console.log(JSON.stringify({type:"welcome"})); process.exit(3)'],
		};
		const host = new AdapterHost(crashed);
		expect((await host.awaitFrame({ type: "welcome" })).type).toBe("welcome");
		const waited = await host.awaitFrame({ type: "done" }).then(
			() => "came",
			() => "failed",
		);
		expect(waited).toBe("failed");
		expect(await host.close()).toBe(3);
	});
});
