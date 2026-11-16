// Server-sent event decoding.

import type { SseItem } from "@nyaaai/contract/shapes/step";
import { Utf8Decoder } from "@nyaaai/contract/text";

////////////////////////////////
//  Classes

/** Decodes split UTF-8; accepts LF and CRLF. */
export class SseDecoder {
	private readonly decoder = new Utf8Decoder();
	private pending = "";
	private data: string[] = [];
	private event: string | undefined;

	push(bytes: Uint8Array): SseItem[] {
		this.pending += this.decoder.push(bytes);
		const items: SseItem[] = [];
		for (let at = this.pending.indexOf("\n"); at !== -1; at = this.pending.indexOf("\n")) {
			const line = this.pending.slice(0, at);
			this.pending = this.pending.slice(at + 1);
			this.line(line.endsWith("\r") ? line.slice(0, -1) : line, items);
		}
		return items;
	}

	/** Flushes unterminated final data. */
	end(): SseItem[] {
		const items = this.push(new Uint8Array());
		this.pending += this.decoder.end();
		if (this.pending.length > 0)
			this.line(this.pending.endsWith("\r") ? this.pending.slice(0, -1) : this.pending, items);
		this.pending = "";
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
		if (field === "event") this.event = value;
		if (field === "data") this.data.push(value);
	}

	private dispatch(items: SseItem[]): void {
		const event = this.event;
		this.event = undefined;
		if (this.data.length === 0) return;
		const data = this.data.join("\n");
		this.data = [];
		if (data === "[DONE]") items.push({ kind: "done" });
		else items.push(event === undefined ? { kind: "data", data } : { kind: "data", data, event });
	}
}
