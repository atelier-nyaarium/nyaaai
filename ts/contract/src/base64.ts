// Padded standard base64.

////////////////////////////////
//  Constants

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PAD = "=";

////////////////////////////////
//  Functions & Helpers

export function base64Encode(bytes: Uint8Array): string {
	let text = "";
	for (let at = 0; at < bytes.length; at += 3) {
		const [first = 0, second = 0, third = 0] = bytes.subarray(at, at + 3);
		const left = bytes.length - at;
		text += ALPHABET[first >> 2];
		text += ALPHABET[((first & 3) << 4) | (second >> 4)];
		text += left > 1 ? ALPHABET[((second & 15) << 2) | (third >> 6)] : PAD;
		text += left > 2 ? ALPHABET[third & 63] : PAD;
	}
	return text;
}

/** Strict padded base64. */
export function base64Decode(text: string): Uint8Array | undefined {
	if (text.length % 4 !== 0) return undefined;
	const padding = text.endsWith(PAD + PAD) ? 2 : text.endsWith(PAD) ? 1 : 0;
	const bytes = new Uint8Array((text.length / 4) * 3 - padding);
	let at = 0;
	for (let from = 0; from < text.length; from += 4) {
		const last = from + 4 === text.length;
		let held = 0;
		for (let place = 0; place < 4; place += 1) {
			const padded = last && place >= 4 - padding;
			const six = padded ? 0 : ALPHABET.indexOf(text.charAt(from + place));
			if (six === -1) return undefined;
			held = (held << 6) | six;
		}
		const count = last ? 3 - padding : 3;
		// Reject stray bits.
		if ((held & ((1 << (8 * (3 - count))) - 1)) !== 0) return undefined;
		for (let place = 0; place < count; place += 1) {
			bytes[at] = (held >> (16 - 8 * place)) & 255;
			at += 1;
		}
	}
	return bytes;
}
