// Transport case sources.

import type { CaseSource } from "../caseAdapter";
import { budgetTransportCases } from "./transportBudget";
import { fixCases } from "./transportFixes";
import { flowCases } from "./transportFlow";
import { httpCases } from "./transportHttp";
import { keyCases } from "./transportKeys";

export const transportCases: CaseSource[] = [
	...flowCases,
	...keyCases,
	...httpCases,
	...fixCases,
	...budgetTransportCases,
];
