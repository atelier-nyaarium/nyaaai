// CLI entry point. Run: bun ts/cli/src/main.ts serve

import { serveStdio } from "@nyaaai/adapter/stdio";
import { COMMANDS } from "./commands";

////////////////////////////////
//  Main

if (import.meta.main) {
	const [name] = process.argv.slice(2);
	if (name !== "serve") {
		console.error("usage:\n  nyaaai serve");
		process.exit(1);
	}
	process.exit(await serveStdio(COMMANDS));
}
