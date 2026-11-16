// Google generateContent wire types.

////////////////////////////////
//  Interfaces & Types

export type Part =
	| { text: string; thought?: true; thoughtSignature?: string }
	| { functionCall: { name: string; args: Record<string, unknown> }; thoughtSignature?: string }
	| { functionResponse: { name: string; response: Record<string, unknown> } };

export interface Content {
	role: "user" | "model";
	parts: Part[];
}

export interface FunctionDeclaration {
	name: string;
	description: string;
	parametersJsonSchema: Record<string, unknown>;
}

export type ThinkingLevel = "MINIMAL" | "LOW" | "MEDIUM" | "HIGH";

/** Deliberately omits sampling and safety settings. */
export interface GenerateContentRequest {
	contents: Content[];
	systemInstruction?: { parts: { text: string }[] };
	tools?: { functionDeclarations: FunctionDeclaration[] }[];
	toolConfig?: { functionCallingConfig: { mode: "ANY"; allowedFunctionNames?: string[] } };
	generationConfig?: {
		responseMimeType?: "application/json";
		responseJsonSchema?: Record<string, unknown>;
		thinkingConfig?: { includeThoughts: true; thinkingLevel?: ThinkingLevel };
		/** Counts thought tokens too. */
		maxOutputTokens?: number;
	};
}
