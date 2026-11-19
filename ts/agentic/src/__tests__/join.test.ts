// The one join of reads that run at once.

import { expect, it } from "bun:test";
import { rejection } from "@nyaaai/testkit/rejection";
import { joinAll, joinSettled, told } from "../kinds/join";

////////////////////////////////
//  Tests

it("joinAll starts every read at once in list order and answers in list order", async () => {
	const started: number[] = [];
	const answers = await joinAll(
		[30, 10, 20].map((wait, at) => async () => {
			started.push(at);
			await new Promise((resolve) => setTimeout(resolve, wait));
			return at;
		}),
	);
	expect(started).toEqual([0, 1, 2]);
	expect(answers).toEqual([0, 1, 2]);
});

it("joinSettled waits for every read, ends none, and answers the first failure in list order", async () => {
	const log: string[] = [];
	const late = (wait: number, name: string, fail?: Error) => async () => {
		log.push(`${name} started`);
		await new Promise((resolve) => setTimeout(resolve, wait));
		log.push(`${name} settled`);
		if (fail !== undefined) throw fail;
		return name;
	};
	const first = new Error("first");
	const second = new Error("second");
	const error = await rejection(joinSettled([late(20, "a", first), late(1, "b", second), late(5, "c")]));
	log.push("answered");
	expect(error).toBe(first);
	expect(log.slice(0, 3)).toEqual(["a started", "b started", "c started"]);
	expect(log[log.length - 1]).toBe("answered");
	expect(log).toContain("a settled");
	expect(await joinSettled([late(5, "x"), late(1, "y")])).toEqual(["x", "y"]);
});

it("told swallows what a listener throws", () => {
	expect(() =>
		told(() => {
			throw new Error("no");
		}),
	).not.toThrow();
	expect(() => told(undefined)).not.toThrow();
});
