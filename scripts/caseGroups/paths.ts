import { z } from "@nyaaai/contract/shape";
import { isInside } from "@nyaaai/process/paths";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { plant, TreeEntry } from "@nyaaai/testkit/tree";
import { pathCases } from "../caseSources/paths";

/** The working directory is shared. */
let queue: Promise<unknown> = Promise.resolve();

function exclusively<T>(run: () => Promise<T>): Promise<T> {
	const turn = queue.then(run, run);
	queue = turn.catch(() => undefined);
	return turn;
}

export const paths = caseGroup("paths", {
	kinds: {
		isInside: kind(
			z.object({
				tree: z.array(TreeEntry),
				checks: z.array(z.object({ child: z.string(), parent: z.string() })),
				/** Tree-rooted relative paths. */
				relative: z.literal(true).exactOptional(),
			}),
			z.object({ results: z.array(z.union([z.boolean(), z.literal("error")])) }),
			async (input, context) => {
				const root = context.tmp();
				plant(root, input.tree);
				const place = (text: string) => (input.relative || text.startsWith("/") ? text : `${root}/${text}`);
				const check = async (): Promise<Array<boolean | "error">> => {
					const results: Array<boolean | "error"> = [];
					for (const held of input.checks) {
						results.push(
							await isInside(place(held.child), place(held.parent)).catch(() => "error" as const),
						);
					}
					return results;
				};
				if (!input.relative) return { results: await check() };
				return {
					results: await exclusively(async () => {
						const started = process.cwd();
						try {
							process.chdir(root);
							return await check();
						} finally {
							process.chdir(started);
						}
					}),
				};
			},
			{ context: true },
		),
	},
	sources: casesFor("paths", pathCases),
});
