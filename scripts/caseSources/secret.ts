// Secret spelling and scrub cases. Every key is made up.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Constants

const KEY = "fake-key-one";
/** Percent and base64 both change it. */
const ODD = "fake/cd+ef=gh";

////////////////////////////////
//  Functions & Helpers

const b64 = (text: string): string => Buffer.from(text).toString("base64").replace(/=+$/, "");

const spells = (name: string, secret: string): CaseSource => ({
	kind: "secret.spellings",
	name,
	input: { secret },
});

const scrubs = (name: string, secret: string, text: string): CaseSource => ({
	kind: "secret.scrub",
	name,
	input: { secret, text },
});

////////////////////////////////
//  Cases

export const secretCases: CaseSource[] = [
	spells("spellings-plain-key", KEY),
	spells("spellings-awkward-key", ODD),
	spells("spellings-one-character", "k"),
	spells("spellings-two-characters", "ab"),
	spells("spellings-three-characters", "abc"),
	spells("spellings-empty", ""),
	spells("spellings-wide-text", "fakeキー"),
	scrubs("scrub-as-is", KEY, `Bad ${KEY} end`),
	scrubs("scrub-twice-in-a-row", KEY, `${KEY}${KEY}`),
	scrubs("scrub-path-encoded-upper-hex", ODD, "x fake%2Fcd%2Bef%3Dgh y"),
	scrubs("scrub-path-encoded-lower-hex", ODD, "x fake%2fcd%2bef%3dgh y"),
	scrubs("scrub-path-encoded-mixed-hex", ODD, "x fake%2fcd%2Bef%3dgh y"),
	scrubs("scrub-path-encoded-letters-keep-their-case", ODD, "x FAKE%2fCD%2bEF%3dGH y"),
	scrubs("scrub-half-encoded-is-not-a-spelling", ODD, "x fake%2fcd+ef=gh y"),
	scrubs("scrub-base64-whole", KEY, `x ${b64(KEY)} y`),
	scrubs("scrub-base64-after-one-byte", KEY, `x ${b64(`a${KEY}-tail`)} y`),
	scrubs("scrub-base64-after-two-bytes", KEY, `x ${b64(`ab${KEY}-tail`)} y`),
	scrubs("scrub-base64-after-three-bytes", KEY, `x ${b64(`abc${KEY}-tail`)} y`),
	scrubs("scrub-base64-of-a-bearer-header", KEY, `dump ${b64(`Bearer ${KEY}`)}`),
	scrubs("scrub-base64-of-a-bearer-header-with-a-tail", KEY, `dump ${b64(`Bearer ${KEY}\r\n`)} end`),
	scrubs("scrub-base64-of-user-and-key", KEY, `dump ${b64(`user:${KEY}`)}`),
	scrubs("scrub-base64-of-user-and-key-with-a-tail", KEY, `dump ${b64(`user:${KEY}:x`)} end`),
	scrubs("scrub-base64-of-the-awkward-key-shifted", ODD, `dump ${b64(`Bearer ${ODD}`)}`),
	scrubs("scrub-base64-of-another-text-is-left", KEY, `dump ${b64("Bearer fake-something-else")}`),
	scrubs("scrub-one-character-key-does-not-reread-the-marker", "k", "key is e here"),
	scrubs("scrub-empty-key-scrubs-nothing", "", "nothing to see"),
	scrubs("scrub-no-match-is-unchanged", KEY, "plain text"),
	scrubs("scrub-the-marker-text-is-not-a-secret", KEY, "[key] stays"),
	scrubs("scrub-each-spelling-in-one-text", ODD, `${ODD} fake%2fcd%2bef%3dgh ${b64(ODD)}`),
	scrubs("scrub-overlapping-spellings-take-the-longest", KEY, `${KEY}${b64(KEY)}`),
	scrubs("scrub-wide-text", "fakeキー", "a fakeキー b"),
	scrubs("scrub-empty-text", KEY, ""),
];
