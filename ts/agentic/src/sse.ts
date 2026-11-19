// Server-sent event decoding.

import type { SseItem } from "@nyaaai/contract/shapes/step";
import { Utf8Decoder, utf8Length } from "@nyaaai/contract/text";

////////////////////////////////
//  Classes

/** Decodes split UTF-8; a line ends at LF, a lone CR is data. */
export class SseDecoder {
	private readonly decoder = new Utf8Decoder();
	private pending = "";
	/** Pending text before this offset holds no LF. */
	private scanned = 0;
	private data: string[] = [];
	private event: string | undefined;
	private pendingBytes = 0;
	private dataBytes = 0;
	private eventBytes = 0;

	/** UTF-8 bytes held: the pending line, and the gathered data and event lines. */
	held(): number {
		return this.pendingBytes + this.dataBytes + this.eventBytes;
	}

	push(bytes: Uint8Array): SseItem[] {
		const text = this.decoder.push(bytes);
		this.pending += text;
		this.pendingBytes += utf8Length(text);
		const items: SseItem[] = [];
		let from = 0;
		for (let at = this.pending.indexOf("\n", this.scanned); at !== -1; at = this.pending.indexOf("\n", from)) {
			const line = this.pending.slice(from, at);
			from = at + 1;
			this.line(line.endsWith("\r") ? line.slice(0, -1) : line, items);
		}
		if (from > 0) {
			this.pending = this.pending.slice(from);
			this.pendingBytes = utf8Length(this.pending);
		}
		this.scanned = this.pending.length;
		return items;
	}

	/** Flushes unterminated final data. */
	end(): SseItem[] {
		const items = this.push(new Uint8Array());
		this.pending += this.decoder.end();
		if (this.pending.length > 0)
			this.line(this.pending.endsWith("\r") ? this.pending.slice(0, -1) : this.pending, items);
		this.pending = "";
		this.pendingBytes = 0;
		this.scanned = 0;
		this.dispatch(items);
		return items;
	}

	private line(line: string, items: SseItem[]): void {
		if (line.length === 0) {
			this.dispatch(items);
			return;
		}
		if (line.startsWith(":")) {
			items.push({ kind: "comment", text: line.slice(1) });
			return;
		}
		const colon = line.indexOf(":");
		const field = colon === -1 ? line : line.slice(0, colon);
		const raw = colon === -1 ? "" : line.slice(colon + 1);
		const value = raw.startsWith(" ") ? raw.slice(1) : raw;
		if (field === "event") {
			this.event = value;
			this.eventBytes = utf8Length(line) + 1;
		}
		if (field === "data") {
			this.data.push(value);
			this.dataBytes += utf8Length(line) + 1;
		}
	}

	private dispatch(items: SseItem[]): void {
		const event = this.event;
		this.event = undefined;
		this.eventBytes = 0;
		if (this.data.length === 0) return;
		const data = this.data.join("\n");
		this.data = [];
		this.dataBytes = 0;
		if (data === "[DONE]") items.push({ kind: "done" });
		else items.push(event === undefined ? { kind: "data", data } : { kind: "data", data, event });
	}
}
