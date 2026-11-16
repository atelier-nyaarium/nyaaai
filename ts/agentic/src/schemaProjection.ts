// Provider JSON schema projections.

////////////////////////////////
//  Interfaces & Types

/** @unknown JSON Schema nodes. */
export interface Schema {
	[key: string]: unknown;
	$schema?: unknown;
	additionalProperties?: unknown;
	anyOf?: unknown;
	const?: unknown;
	description?: unknown;
	enum?: unknown;
	format?: unknown;
	items?: unknown;
	oneOf?: unknown;
	properties?: unknown;
	required?: unknown;
	type?: unknown;
}

////////////////////////////////
//  Functions & Helpers

function isSchema(value: unknown): value is Schema {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}

function removeFormat(node: unknown): void {
	if (Array.isArray(node)) {
		node.forEach(removeFormat);
		return;
	}
	if (!isSchema(node)) return;
	if (node.format === "uri" || node.format === "uri-reference") {
		delete node.format;
	}
	for (const key in node) {
		removeFormat(node[key]);
	}
}

/** Removes unsupported formats and closes the root object. */
export function cleanSchema(schema: Schema): Schema {
	const jsonSchema = structuredClone(schema);
	if (jsonSchema.$schema) delete jsonSchema.$schema;
	if (jsonSchema.type === "object") {
		jsonSchema.additionalProperties = false;
		removeFormat(jsonSchema);
	}
	return jsonSchema;
}

function requiredOf(schema: Schema): Set<unknown> {
	return new Set(Array.isArray(schema.required) ? schema.required : []);
}

function branchesOf(schema: Schema): Schema[] {
	const branches = schema.anyOf ?? schema.oneOf;
	return Array.isArray(branches) ? branches.filter(isSchema) : [];
}

function acceptsNull(schema: Schema): boolean {
	const { type } = schema;
	if (type === "null" || (Array.isArray(type) && type.includes("null"))) return true;
	const branches = branchesOf(schema);
	if (branches.length > 0) return branches.some(acceptsNull);
	if (Array.isArray(schema.enum)) return schema.enum.includes(null);
	if ("const" in schema) return schema.const === null;
	return type === undefined;
}

function orNull(schema: Schema): Schema {
	if (typeof schema.type === "string" && !("enum" in schema) && !("const" in schema))
		return { ...schema, type: [schema.type, "null"] };
	const { description, ...rest } = schema;
	return { anyOf: [rest, { type: "null" }], ...(description === undefined ? {} : { description }) };
}

/** Models fill all fields. */
export function nullableOptionals(schema: Schema): Schema {
	const out: Schema = { ...schema };
	if (isSchema(schema.items)) out.items = nullableOptionals(schema.items);
	for (const key of ["anyOf", "oneOf"]) {
		const branches = schema[key];
		if (Array.isArray(branches))
			out[key] = branches.map((branch) => (isSchema(branch) ? nullableOptionals(branch) : branch));
	}
	if (!isSchema(schema.properties)) return out;
	const required = requiredOf(schema);
	out.properties = Object.fromEntries(
		Object.entries(schema.properties).map(([key, field]) => {
			if (!isSchema(field)) return [key, field];
			const inner = nullableOptionals(field);
			return [key, required.has(key) || acceptsNull(inner) ? inner : orNull(inner)];
		}),
	);
	out.required = Object.keys(schema.properties);
	return out;
}

/** Drops each null `nullableOptionals` allowed in a tool call; a union node passes as sent. */
export function withoutNulls(schema: Schema, value: unknown): unknown {
	if (Array.isArray(value)) {
		const { items } = schema;
		return isSchema(items) ? value.map((each) => withoutNulls(items, each)) : value;
	}
	if (!isSchema(value) || !isSchema(schema.properties)) return value;
	const properties = schema.properties;
	const required = requiredOf(schema);
	// Preserve own `__proto__` keys.
	return Object.fromEntries(
		Object.entries(value).flatMap(([key, field]) => {
			const inner = Object.hasOwn(properties, key) ? properties[key] : undefined;
			if (!isSchema(inner)) return [[key, field]];
			if (field === null && !required.has(key) && !acceptsNull(inner)) return [];
			return [[key, withoutNulls(inner, field)]];
		}),
	);
}

/** Closes objects, uses `anyOf`; refuses what strict mode cannot carry. */
function closed(schema: Schema): Schema {
	for (const key of ["$ref", "$defs", "prefixItems"])
		if (key in schema)
			throw new Error(`A strict json_schema cannot carry \`${key}\`; reshape the answer without it.`);
	if (isSchema(schema.additionalProperties))
		throw new Error("A strict json_schema cannot carry an open record; name its keys as properties.");
	const required = requiredOf(schema);
	const optional = Object.keys(isSchema(schema.properties) ? schema.properties : {}).find(
		(key) => !required.has(key),
	);
	if (optional !== undefined)
		throw new Error(
			`A strict json_schema cannot carry an optional field \`${optional}\`; make it \`.nullable()\`, which is what strict mode sends.`,
		);
	const out: Schema = { ...schema };
	if (isSchema(schema.items)) out.items = closed(schema.items);
	const branches = branchesOf(schema);
	if (branches.length > 0) {
		delete out.oneOf;
		out.anyOf = branches.map(closed);
	}
	if (isSchema(schema.properties)) {
		out.properties = Object.fromEntries(
			Object.entries(schema.properties).map(([key, field]) => [key, isSchema(field) ? closed(field) : field]),
		);
		out.additionalProperties = false;
	}
	return out;
}

/** What a strict `json_schema` sends. An empty field is `.nullable()`, as the model answers it. */
export function strictSchema(schema: Schema): Schema {
	return closed(cleanSchema(schema));
}
