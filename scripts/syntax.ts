import { API } from "../ts/node_modules/typescript/dist/api/async/api.js";
import { createVirtualFileSystem } from "../ts/node_modules/typescript/dist/api/fs.js";
import type { Node, SourceFile } from "../ts/node_modules/typescript/dist/ast/index.js";

export * from "../ts/node_modules/typescript/dist/ast/index.js";

const FOLDER = "/syntax";

/** Parses texts with the TypeScript compiler's parser, keyed as given. */
export async function parseSources(texts: Record<string, string>): Promise<Map<string, SourceFile>> {
	const keys = Object.keys(texts);
	const names = keys.map((key, at) => `${at}${key.endsWith(".tsx") ? ".tsx" : ".ts"}`);
	const files: Record<string, string> = {
		[`${FOLDER}/tsconfig.json`]: JSON.stringify({
			files: names,
			compilerOptions: { noLib: true, noResolve: true, types: [], jsx: "preserve" },
		}),
	};
	keys.forEach((key, at) => {
		files[`${FOLDER}/${names[at]}`] = texts[key] as string;
	});
	const api = new API({ fs: createVirtualFileSystem(files), cwd: FOLDER });
	try {
		const snapshot = await api.updateSnapshot({ openProject: `${FOLDER}/tsconfig.json` });
		const project = snapshot.getProject(`${FOLDER}/tsconfig.json`);
		if (!project) throw new Error("the parser opened no project");
		const parsed = new Map<string, SourceFile>();
		for (const [at, key] of keys.entries()) {
			const file = await project.program.getSourceFile(`${FOLDER}/${names[at]}`);
			if (!file) throw new Error(`the parser read nothing of ${key}`);
			parsed.set(key, file);
		}
		return parsed;
	} finally {
		await api.close();
	}
}

/** Every node under one, in source order. */
export function descendants(root: Node): Node[] {
	const found: Node[] = [];
	const visit = (node: Node) => {
		found.push(node);
		node.forEachChild(visit);
	};
	visit(root);
	return found;
}
