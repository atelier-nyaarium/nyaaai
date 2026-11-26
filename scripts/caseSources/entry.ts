// Provider entry cases.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Helpers

const parse = (name: string, value: unknown): CaseSource => ({ kind: "entry.parse", name, input: { value } });

const list = (name: string, value: unknown): CaseSource => ({ kind: "entry.parseList", name, input: { value } });

const origins = (name: string, rows: Array<[string | null, string | null]>): CaseSource => ({
	kind: "entry.sameOrigin",
	name,
	input: { rows: rows.map(([left, right]) => ({ left, right })) },
});

const ollama = (url: unknown) => ({ id: "box", kind: "ollama", label: "Lab", url });

const compatible = (url: unknown) => ({ id: "api", kind: "compatible", label: "Lab", url });

/** A parse of a URL for each local kind, as a case per kind. */
const localUrl = (name: string, url: unknown): CaseSource[] =>
	["ollama", "lmstudio", "llamacpp"].map((kind) => parse(`${name}-${kind}`, { id: "box", kind, label: "Lab", url }));

const BASE = { id: "or1", kind: "openrouter", label: "OpenRouter" };

const binding = (name: string, rows: unknown[]): CaseSource => ({ kind: "entry.binding", name, input: { rows } });

const compatibleFields = (url: string) => ({ kind: "compatible", label: "Lab", url });

////////////////////////////////
//  Cases

/** Ids of each shape `isEntryId` judges. */
const IDS: unknown[] = [
	"a",
	"A-b_c",
	"3f2a1b4c-5d6e-4f70-8a9b-0c1d2e3f4a5b",
	"x".repeat(64),
	"x".repeat(65),
	"",
	"a b",
	"a.b",
	"a/b",
	"a:b",
	"\u{e9}",
	"abc\n",
	"\nabc",
	" abc",
	"__proto__",
	"constructor",
	"-",
	"_",
	"0",
	"\u{ff21}",
	42,
	null,
	true,
	{},
	["a"],
];

export const entryCases: CaseSource[] = [
	// Whole entries, one per kind.
	parse("openrouter", BASE),
	parse("opencode-go", { id: "go1", kind: "opencode", label: "Go", plan: "go" }),
	parse("opencode-go-extra", { id: "go1", kind: "opencode", label: "Go", plan: "go", extraUsage: true }),
	parse("opencode-go-extra-false", { id: "go1", kind: "opencode", label: "Go", plan: "go", extraUsage: false }),
	parse("opencode-go-extra-text", { id: "go1", kind: "opencode", label: "Go", plan: "go", extraUsage: "yes" }),
	parse("opencode-go-extra-null", { id: "go1", kind: "opencode", label: "Go", plan: "go", extraUsage: null }),
	parse("opencode-payg", { id: "pay", kind: "opencode", label: "Pay", plan: "payg" }),
	parse("opencode-payg-extra-dropped", { id: "pay", kind: "opencode", label: "Pay", plan: "payg", extraUsage: true }),
	parse("opencode-no-plan", { id: "go1", kind: "opencode", label: "Go" }),
	parse("opencode-plan-null", { id: "go1", kind: "opencode", label: "Go", plan: null }),
	parse("opencode-plan-unknown", { id: "go1", kind: "opencode", label: "Go", plan: "team" }),
	parse("opencode-plan-case", { id: "go1", kind: "opencode", label: "Go", plan: "GO" }),
	parse("opencode-plan-number", { id: "go1", kind: "opencode", label: "Go", plan: 1 }),
	parse("ollama-no-url", { id: "box", kind: "ollama", label: "Lab" }),
	parse("lmstudio-no-url", { id: "box", kind: "lmstudio", label: "Lab" }),
	parse("llamacpp-no-url", { id: "box", kind: "llamacpp", label: "Lab" }),
	parse("compatible-full", { id: "api", kind: "compatible", label: "Lab", url: "https://lab.local:8443/v1" }),

	// Fields that belong to another kind are dropped.
	parse("openrouter-drops-plan", { ...BASE, plan: "go" }),
	parse("openrouter-drops-url", { ...BASE, url: "http://lab.local:1234" }),
	parse("openrouter-drops-extra-usage", { ...BASE, extraUsage: true }),
	parse("unknown-keys-dropped", { ...BASE, color: "red", nested: { a: 1 } }),

	// Not an object.
	parse("null", null),
	parse("text", "openrouter"),
	parse("number", 7),
	parse("true", true),
	parse("array", [BASE]),
	parse("empty-object", {}),

	// Every field missing, mistyped or odd.
	parse("id-missing", { kind: "openrouter", label: "Lab" }),
	parse("id-number", { ...BASE, id: 7 }),
	parse("id-null", { ...BASE, id: null }),
	parse("id-empty", { ...BASE, id: "" }),
	parse("id-64", { ...BASE, id: "x".repeat(64) }),
	parse("id-65", { ...BASE, id: "x".repeat(65) }),
	parse("id-space", { ...BASE, id: "a b" }),
	parse("id-trailing-newline", { ...BASE, id: "abc\n" }),
	parse("id-padded", { ...BASE, id: " abc " }),
	parse("id-proto", { ...BASE, id: "__proto__" }),
	parse("id-proto-like", { ...BASE, id: "__proto" }),
	parse("id-constructor", { ...BASE, id: "constructor" }),
	parse("id-uuid", { ...BASE, id: "3f2a1b4c-5d6e-4f70-8a9b-0c1d2e3f4a5b" }),
	parse("id-non-ascii", { ...BASE, id: "\u{e9}" }),
	parse("kind-missing", { id: "or1", label: "Lab" }),
	parse("kind-unknown", { ...BASE, kind: "gemini" }),
	parse("kind-case", { ...BASE, kind: "OpenRouter" }),
	parse("kind-number", { ...BASE, kind: 1 }),
	parse("kind-null", { ...BASE, kind: null }),
	parse("label-missing", { id: "or1", kind: "openrouter" }),
	parse("label-number", { ...BASE, label: 5 }),
	parse("label-null", { ...BASE, label: null }),
	parse("label-empty", { ...BASE, label: "" }),
	parse("label-blank", { ...BASE, label: " \t\n " }),
	parse("label-trimmed", { ...BASE, label: "  Lab  " }),
	parse("label-trims-ecma-space", { ...BASE, label: "\u{feff}\u{a0}\u{2028}Lab\u{3000}\u{1680}" }),
	parse("label-keeps-zero-width", { ...BASE, label: "\u{200b}Lab\u{200b}" }),
	parse("label-only-zero-width", { ...BASE, label: "\u{200b}" }),
	parse("label-inner-space-kept", { ...BASE, label: "Lab  box" }),
	parse("label-120", { ...BASE, label: "L".repeat(120) }),
	parse("label-121", { ...BASE, label: "L".repeat(121) }),
	parse("label-120-after-trim", { ...BASE, label: ` ${"L".repeat(120)} ` }),
	parse("label-60-astral", { ...BASE, label: "\u{1f600}".repeat(60) }),
	parse("label-61-astral", { ...BASE, label: "\u{1f600}".repeat(61) }),

	// URL rules of the local kinds: an origin, nothing else.
	...localUrl("url-origin", "http://lab.local:11434"),
	...localUrl("url-slash", "http://lab.local:11434/"),
	...localUrl("url-padded", "  http://lab.local:11434/  "),
	...localUrl("url-https", "https://lab.local"),
	...localUrl("url-default-port", "http://lab.local:80"),
	...localUrl("url-default-port-https", "https://lab.local:443/"),
	...localUrl("url-other-port-kept", "https://lab.local:80"),
	...localUrl("url-upper-case", "HTTP://LAB.Local:11434"),
	...localUrl("url-ipv4", "http://192.168.1.20:8080"),
	...localUrl("url-ipv4-hex", "http://0x7f.1:8080"),
	...localUrl("url-ipv6", "http://[::1]:8080"),
	...localUrl("url-ipv6-long", "http://[2001:DB8:0:0:0:0:0:1]/"),
	...localUrl("url-trailing-dot", "http://lab.local.:11434"),
	...localUrl("url-empty-query", "http://lab.local:11434/?"),
	...localUrl("url-empty-fragment", "http://lab.local:11434/#"),
	...localUrl("url-dot-path", "http://lab.local:11434/."),
	...localUrl("url-dot-dot-path", "http://lab.local:11434/a/.."),
	...localUrl("url-backslash", "http:\\\\lab.local:11434\\"),
	...localUrl("url-no-slashes", "http:lab.local:11434"),
	...localUrl("url-path", "http://lab.local:11434/v1"),
	...localUrl("url-path-slash", "http://lab.local:11434/v1/"),
	...localUrl("url-double-slash", "http://lab.local:11434//"),
	...localUrl("url-query", "http://lab.local:11434/?a=1"),
	...localUrl("url-fragment", "http://lab.local:11434/#a"),
	...localUrl("url-user", "http://user@lab.local:11434"),
	...localUrl("url-user-password", "http://user:pass@lab.local:11434"),
	...localUrl("url-empty-user", "http://@lab.local:11434"),
	...localUrl("url-empty-user-password", "http://:@lab.local:11434"),
	...localUrl("url-password-only", "http://:pass@lab.local:11434"),
	...localUrl("url-ftp", "ftp://lab.local"),
	...localUrl("url-ws", "ws://lab.local"),
	...localUrl("url-file", "file:///lab"),
	...localUrl("url-no-scheme", "lab.local:11434"),
	...localUrl("url-bare-host", "lab.local"),
	...localUrl("url-scheme-only", "http://"),
	...localUrl("url-empty-host", "http:///"),
	...localUrl("url-empty", ""),
	...localUrl("url-blank", "   "),
	...localUrl("url-null", null),
	...localUrl("url-number", 11434),
	...localUrl("url-object", { href: "http://lab.local" }),
	...localUrl("url-port-too-big", "http://lab.local:99999"),
	...localUrl("url-port-text", "http://lab.local:port"),
	...localUrl("url-empty-port", "http://lab.local:"),
	...localUrl("url-space-in-host", "http://lab local"),
	...localUrl("url-non-ascii-host", "http://l\u{e4}b.local"),
	...localUrl("url-percent-host", "http://l%61b.local"),
	...localUrl("url-bad-ipv6", "http://[::1"),
	...localUrl("url-tab-inside", "http://lab\t.local:11434"),
	parse("url-on-other-kinds-ignored", { id: "go1", kind: "opencode", label: "Go", plan: "go", url: "javascript:1" }),

	// The compatible kind: a URL is required, its path kept, trailing slashes dropped.
	parse("compatible-missing", { id: "api", kind: "compatible", label: "Lab" }),
	parse("compatible-null", compatible(null)),
	parse("compatible-empty", compatible("")),
	parse("compatible-origin", compatible("https://api.example.com")),
	parse("compatible-origin-slash", compatible("https://api.example.com/")),
	parse("compatible-path", compatible("https://api.example.com/v1")),
	parse("compatible-path-slashes", compatible("https://api.example.com/v1///")),
	parse("compatible-path-deep", compatible("https://api.example.com/a/b/c/")),
	parse("compatible-padded", compatible("  https://api.example.com/v1  ")),
	parse("compatible-default-port", compatible("https://api.example.com:443/v1")),
	parse("compatible-port", compatible("http://lab.local:8080/v1")),
	parse("compatible-upper-host", compatible("HTTPS://API.Example.COM/V1")),
	parse("compatible-space-path", compatible("https://api.example.com/my path/")),
	parse("compatible-escapes-kept", compatible("https://api.example.com/a%2Fb/%7e")),
	parse("compatible-bad-escape-kept", compatible("https://api.example.com/a%zz")),
	parse("compatible-non-ascii-path", compatible("https://api.example.com/caf\u{e9}")),
	parse("compatible-astral-path", compatible("https://api.example.com/\u{1f600}")),
	parse("compatible-special-path", compatible('https://api.example.com/a"b<c>d^e`f{g}h|i')),
	parse("compatible-dot-segments", compatible("https://api.example.com/a/./b/../v1/")),
	parse("compatible-dot-dot-above-root", compatible("https://api.example.com/../../v1")),
	parse("compatible-encoded-dots", compatible("https://api.example.com/a/%2e%2E/v1")),
	parse("compatible-double-slash", compatible("https://api.example.com//v1//")),
	parse("compatible-backslash", compatible("https://api.example.com\\v1\\")),
	parse("compatible-empty-query", compatible("https://api.example.com/v1?")),
	parse("compatible-empty-fragment", compatible("https://api.example.com/v1#")),
	parse("compatible-query", compatible("https://api.example.com/v1?key=1")),
	parse("compatible-fragment", compatible("https://api.example.com/v1#top")),
	parse("compatible-user", compatible("https://user:pass@api.example.com/v1")),
	parse("compatible-ftp", compatible("ftp://api.example.com/v1")),
	parse("compatible-no-scheme", compatible("api.example.com/v1")),
	parse("compatible-non-ascii-host", compatible("https://api.ex\u{e4}mple.com/v1")),
	parse("compatible-percent-host", compatible("https://api.ex%61mple.com/v1")),
	parse("compatible-ipv6", compatible("http://[::1]:8080/v1/")),
	parse("compatible-number", compatible(5)),
	parse("local-url-on-ollama-null", ollama(null)),

	// A settings list: what reads, in order.
	list("list-empty", []),
	list("list-null", null),
	list("list-object", { 0: BASE }),
	list("list-text", "openrouter"),
	list("list-one", [BASE]),
	list("list-order", [
		{ id: "b", kind: "openrouter", label: "B" },
		{ id: "a", kind: "opencode", label: "A", plan: "go" },
		{ id: "c", kind: "ollama", label: "C", url: "http://lab.local:11434/" },
	]),
	list("list-drops-invalid", [BASE, 7, null, "x", [], { id: "bad id", kind: "openrouter", label: "x" }, BASE]),
	list("list-first-duplicate-wins", [
		{ id: "a", kind: "openrouter", label: "First" },
		{ id: "a", kind: "opencode", label: "Second", plan: "go" },
	]),
	list("list-invalid-first-duplicate", [
		{ id: "a", kind: "opencode", label: "Broken" },
		{ id: "a", kind: "openrouter", label: "Whole" },
	]),
	list("list-duplicate-after-trim", [
		{ id: "a", kind: "openrouter", label: " One " },
		{ id: "a", kind: "openrouter", label: "One" },
	]),
	list("list-proto-id-dropped", [{ id: "__proto__", kind: "openrouter", label: "x" }, BASE]),
	list("list-ids-are-case-sensitive", [
		{ id: "a", kind: "openrouter", label: "x" },
		{ id: "A", kind: "openrouter", label: "x" },
	]),

	// Sameness of origin.
	origins("origin-default-ports", [
		["http://lab.local", "http://lab.local:80"],
		["https://lab.local", "https://lab.local:443/"],
		["http://lab.local", "http://lab.local:443"],
		["https://lab.local", "https://lab.local:80"],
		["http://lab.local:81", "http://lab.local:81/"],
		["http://lab.local:81", "http://lab.local:82"],
	]),
	origins("origin-scheme", [
		["http://lab.local", "https://lab.local"],
		["HTTP://lab.local", "http://lab.local"],
	]),
	origins("origin-case", [
		["http://LAB.local", "http://lab.LOCAL"],
		["http://Lab.local:8080", "http://lab.local:8080"],
	]),
	origins("origin-slash-and-path", [
		["http://lab.local", "http://lab.local/"],
		["http://lab.local/a", "http://lab.local/b"],
		["http://lab.local/a/b/c", "http://lab.local"],
		["http://lab.local/v1/", "http://lab.local/v2"],
	]),
	origins("origin-padded", [
		["  http://lab.local  ", "http://lab.local"],
		["\thttp://lab.local", "http://lab.local"],
	]),
	origins("origin-empty-query-and-fragment", [
		["http://lab.local/?", "http://lab.local"],
		["http://lab.local/#", "http://lab.local"],
	]),
	origins("origin-query-refused", [
		["http://lab.local/?a=1", "http://lab.local/?a=1"],
		["http://lab.local/#a", "http://lab.local/#a"],
		["http://lab.local/?a=1", "http://lab.local"],
	]),
	origins("origin-user-info-refused", [
		["http://user@lab.local", "http://user@lab.local"],
		["http://user:pass@lab.local", "http://lab.local"],
		["http://lab.local", "http://user:pass@lab.local"],
	]),
	origins("origin-other-schemes-refused", [
		["ftp://lab.local", "ftp://lab.local"],
		["ws://lab.local", "ws://lab.local"],
		["file:///lab", "file:///lab"],
		["javascript:1", "javascript:1"],
	]),
	origins("origin-garbage", [
		["not a url", "not a url"],
		["", ""],
		["   ", "   "],
		["http://", "http://"],
		["lab.local", "lab.local"],
		["http://lab local", "http://lab local"],
		["http://lab.local:99999", "http://lab.local:99999"],
		["http://[::1", "http://[::1"],
	]),
	origins("origin-absent", [
		[null, null],
		[null, "http://lab.local"],
		["http://lab.local", null],
	]),
	origins("origin-hosts", [
		["http://192.168.1.20:8080", "http://192.168.1.20:8080/x"],
		["http://[::1]:8080", "http://[0:0:0:0:0:0:0:1]:8080"],
		["http://0x7f.1", "http://127.0.0.1"],
		["http://lab.local.", "http://lab.local"],
		["http://lab.local.", "http://LAB.local./"],
	]),
	origins("origin-non-ascii-refused", [
		["http://l\u{e4}b.local", "http://l\u{e4}b.local"],
		["http://l%61b.local", "http://lab.local"],
	]),

	// Ids and secret keys.
	{ kind: "entry.id", name: "ids", input: { values: IDS } },
	{
		kind: "entry.secretKey",
		name: "keys",
		input: { ids: ["a", "3f2a1b4c-5d6e-4f70-8a9b-0c1d2e3f4a5b", "x.y", ""], namespace: "app" },
	},
	{ kind: "entry.secretKey", name: "keys-other-namespace", input: { ids: ["a"], namespace: "tool.v2" } },
	{ kind: "entry.secretKey", name: "keys-empty-namespace", input: { ids: ["a"], namespace: "" } },

	// What a key is for.
	binding("binding-a-key-kind-at-its-own-address", [
		{ kind: "openrouter", label: "OpenRouter" },
		{ kind: "openrouter", label: "Other", url: "https://elsewhere.example.test" },
	]),
	binding("binding-the-plan-and-its-default", [
		{ kind: "opencode", label: "Go", plan: "go" },
		{ kind: "opencode", label: "Pay", plan: "payg" },
		{ kind: "opencode", label: "Go" },
		{ kind: "opencode", label: "Go", plan: "go", extraUsage: true },
	]),
	binding("binding-the-origin-of-a-url-with-a-path-a-port-or-a-trailing-slash", [
		compatibleFields("https://api.example.test/v1/chat"),
		compatibleFields("https://api.example.test:8443/v1"),
		compatibleFields("https://api.example.test:443/v1"),
		compatibleFields("http://api.example.test:80"),
		compatibleFields("https://API.Example.TEST/"),
		compatibleFields("https://api.example.test/v1/"),
		compatibleFields("http://api.example.test."),
	]),
	binding("binding-ip-literals", [
		compatibleFields("http://[::1]:8000/v1"),
		compatibleFields("http://[0:0:0:0:0:0:0:1]:8000"),
		compatibleFields("http://[::ffff:192.168.1.5]:8000"),
		compatibleFields("http://192.168.1.5:8000"),
		compatibleFields("http://0x7f.1"),
	]),
	binding("binding-a-url-the-entry-rules-refuse-binds-none", [
		compatibleFields("https://api.example.test/v1?tenant=a"),
		compatibleFields("https://api.example.test/v1#a"),
		compatibleFields("https://user:pw@api.example.test/v1"),
		compatibleFields("ftp://api.example.test"),
		compatibleFields("not a url"),
		{ kind: "compatible", label: "Lab" },
	]),
	binding("binding-a-kind-that-takes-no-key-binds-none", [
		{ kind: "ollama", label: "Lab" },
		{ kind: "ollama", label: "Lab", url: "http://lab.local:11434" },
		{ kind: "lmstudio", label: "Lab", url: "http://lab.local:1234" },
		{ kind: "llamacpp", label: "Lab", url: "http://lab.local:9000" },
	]),
	binding("binding-a-change-of-kind-or-plan-changes-it", [
		{ kind: "openrouter", label: "Lab" },
		compatibleFields("https://openrouter.ai/api/v1"),
		{ kind: "opencode", label: "Lab", plan: "go" },
		{ kind: "opencode", label: "Lab", plan: "payg" },
	]),
	{
		kind: "entry.keyFor",
		name: "for-a-host-else-a-label-else-a-kind-name",
		input: {
			bindings: [
				"compatible https://api.example.test:8443",
				"compatible http://[::1]:8000",
				"openrouter",
				"opencode go",
				"compatible",
				"ollama",
				"team plan",
				"",
				"compatible not-a-url",
			],
			kinds: ["openrouter", "opencode", "compatible"],
		},
	},
	{
		kind: "entry.keyFor",
		name: "for-a-kind-the-table-does-not-hold-is-its-name",
		input: { bindings: ["openrouter", "opencode payg", "compatible https://api.example.test"], kinds: [] },
	},
	{
		kind: "entry.boundKey",
		name: "bound-the-exact-text",
		input: {
			rows: [
				{ fields: { kind: "openrouter", label: "OpenRouter" }, key: "fake-key-one" },
				{ fields: { kind: "opencode", label: "Go" }, key: "fake-key-two" },
				{ fields: compatibleFields("https://api.example.test:8443/v1/"), key: "fake-key-three" },
				{ fields: { kind: "openrouter", label: "OpenRouter" }, key: 'fake "quoted" \\ key\n\u{e9}' },
				{ fields: { kind: "openrouter", label: "OpenRouter" }, key: "" },
			],
		},
	},
	{
		kind: "entry.boundKey",
		name: "bound-refused-for-fields-that-bind-no-key",
		input: {
			rows: [
				{ fields: { kind: "ollama", label: "Lab" }, key: "fake-key-one" },
				{ fields: compatibleFields("https://api.example.test/v1?x=1"), key: "fake-key-one" },
			],
		},
	},
	{
		kind: "entry.readBoundKey",
		name: "read-the-stored-object",
		input: {
			texts: [
				'{"v":1,"for":"openrouter","key":"fake-key-one"}',
				'{"v":1,"for":"compatible https://api.example.test:8443","key":"fake \\"quoted\\" \\\\ key\\n\\u00e9"}',
				'{"key":"fake-key-one","for":"openrouter","v":1}',
				'{"v":1.0,"for":"openrouter","key":"fake-key-one"}',
				' {"v":1,"for":"","key":""} ',
			],
		},
	},
	{
		kind: "entry.readBoundKey",
		name: "read-anything-else-is-none",
		input: {
			texts: [
				"fake-key-one",
				"",
				"null",
				'"fake-key-one"',
				"[1]",
				'{"key":"fake-key-one"}',
				'{"v":2,"for":"openrouter","key":"fake-key-one"}',
				'{"v":"1","for":"openrouter","key":"fake-key-one"}',
				'{"v":1,"for":"openrouter","key":7}',
				'{"v":1,"for":null,"key":"fake-key-one"}',
				'{"v":1,"for":"openrouter","key":"fake-key-one","extra":true}',
				'{"v":1,"for":"openrouter","key":"fake-key-one"',
			],
		},
	},
	{
		kind: "entry.travel",
		name: "travel-https-or-http-to-this-network",
		input: {
			urls: [
				"https://api.example.test/v1",
				"https://8.8.8.8",
				"http://api.example.test/v1",
				"http://8.8.8.8:8000",
				"http://127.0.0.1:8000",
				"http://localhost:8000",
				"http://lab.local:8000",
				"http://labhost",
				"http://192.168.1.5",
				"http://[::1]:8000",
				"http://[fd00::1]",
				"http://[::ffff:192.168.1.5]",
				"http://[::ffff:127.0.0.1]",
				"http://[::ffff:8.8.8.8]",
				"http://[2001:db8::1]",
				"http://10.example.test",
				"HTTP://LAB.LOCAL",
			],
		},
	},
	{
		kind: "entry.travel",
		name: "travel-no-other-scheme-and-no-url-that-does-not-read",
		input: {
			urls: [
				"ws://lab.local",
				"wss://api.example.test",
				"ftp://lab.local",
				"file:///x",
				"not a url",
				"",
				"http://",
			],
		},
	},
];
