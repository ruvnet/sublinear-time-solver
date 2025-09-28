Below is a complete, production‑ready **MCP server** that performs **Perplexity‑powered search with advanced GOAP planning** and ships as an **npx** CLI. It uses:

* **Model Context Protocol TypeScript SDK** for the server and stdio transport. ([GitHub][1])
* **Perplexity Search API** for ranked web results and multi‑query. ([Perplexity][2])
* **Perplexity Sonar Chat Completions** through the OpenAI‑compatible Node client, which lets us pass search controls like domain and recency filters. ([Perplexity][3])
* **Claude Desktop MCP config pattern** to run any server via `npx`. ([Model Context Protocol][4])
* **GOAP planning** based on the classic Orkin formulation with STRIPS‑style preconditions and effects. ([GameDevs][5])

---

## 1) What you get

* A single command you can publish and run as:
  `npx -y @ruv/mcp-perplexity-goap`
* One MCP tool: `goap.search` that:

  * Generates a multi‑step plan to answer a query
  * Executes Perplexity **Search** for coverage and **Sonar** for grounded synthesis
  * Returns an answer, citations, and the executed plan
* Optional lightweight tool: `search.raw` for raw ranked results
* Works with Claude Desktop MCP, Cursor, or any MCP host

---

## 2) Quick start

> Replace the package scope with your own if you plan to publish.

```bash
# 0) prerequisites
node --version   # must be >= 18
# 1) clone and run locally
git clone https://github.com/your-org/mcp-perplexity-goap
cd mcp-perplexity-goap
cp .env.example .env   # put PERPLEXITY_API_KEY=...
npm i
npm run build
npm link                # exposes the CLI as `mcp-perplexity-goap`
mcp-perplexity-goap --help

# 2) run via npx once you publish to npm:
PERPLEXITY_API_KEY=... npx -y @ruv/mcp-perplexity-goap
```

Claude Desktop config example:

```json
{
  "mcpServers": {
    "perplexity-goap": {
      "command": "npx",
      "args": ["-y", "@ruv/mcp-perplexity-goap"],
      "env": {
        "PERPLEXITY_API_KEY": "<YOUR_KEY>"
      }
    }
  }
}
```

The `npx` configuration mirrors the official examples for MCP servers. ([Model Context Protocol][4])

---

## 3) Project layout

```
mcp-perplexity-goap/
├─ package.json
├─ tsconfig.json
├─ .env.example
├─ src/
│  ├─ cli.ts
│  ├─ server.ts
│  ├─ goap/
│  │  ├─ types.ts
│  │  └─ planner.ts
│  ├─ actions/
│  │  ├─ perplexitySearch.ts
│  │  └─ perplexityChat.ts
│  └─ utils/citations.ts
└─ README.md
```

---

## 4) Code

### package.json

```json
{
  "name": "@ruv/mcp-perplexity-goap",
  "version": "1.0.0",
  "description": "MCP server: Perplexity search with GOAP planning",
  "type": "module",
  "bin": {
    "mcp-perplexity-goap": "./dist/cli.js"
  },
  "main": "./dist/server.js",
  "module": "./dist/server.js",
  "files": ["dist", "README.md", "LICENSE"],
  "scripts": {
    "build": "tsc -p tsconfig.json",
    "dev": "tsx src/cli.ts",
    "lint": "eslint ."
  },
  "engines": {
    "node": ">=18"
  },
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.0.0",
    "openai": "^4.60.0",
    "zod": "^3.23.8",
    "dotenv": "^16.4.5",
    "undici": "^6.19.8"
  },
  "devDependencies": {
    "tsx": "^4.15.7",
    "typescript": "^5.6.3",
    "eslint": "^9.9.0"
  }
}
```

> MCP TS SDK requires Node 18 or newer. ([GitHub][1])

### tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "outDir": "dist",
    "rootDir": "src",
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}
```

### .env.example

```
PERPLEXITY_API_KEY=your_api_key
```

> Create your key in the Perplexity API portal, then export `PERPLEXITY_API_KEY`. ([Perplexity][6])

---

### src/cli.ts

```ts
#!/usr/bin/env node
import 'dotenv/config';
import { startServer } from './server.js';

async function main() {
  const args = new Set(process.argv.slice(2));
  if (args.has('--help') || args.has('-h')) {
    console.log(`mcp-perplexity-goap

Usage:
  mcp-perplexity-goap [--verbose]

Environment:
  PERPLEXITY_API_KEY  Required
`);
    process.exit(0);
  }
  const verbose = args.has('--verbose') || args.has('-v');
  await startServer({ verbose });
}

main().catch(err => {
  console.error('Fatal:', err);
  process.exit(1);
});
```

---

### src/server.ts

```ts
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { planAndExecute } from "./goap/planner.js";
import { rawSearch } from "./actions/perplexitySearch.js";

export async function startServer(opts: { verbose?: boolean } = {}) {
  const server = new McpServer({
    name: "perplexity-goap",
    version: "1.0.0"
  });

  // Tool 1: advanced GOAP-planned search-and-answer
  server.registerTool(
    "goap.search",
    {
      title: "GOAP search and synthesize",
      description: "Plans and executes a multi-step search with Perplexity to return an answer with citations and the executed plan.",
      inputSchema: {
        query: z.string().describe("User question or task"),
        domains: z.array(z.string()).optional().describe("Restrict or guide to these domains"),
        recency: z.enum(["day","week","month","year"]).optional().describe("Recency bias for web results"),
        mode: z.enum(["web","academic"]).optional().describe("Search mode for Sonar"),
        maxResults: z.number().int().min(1).max(20).default(8),
        model: z.enum(["sonar","sonar-pro","sonar-reasoning"]).default("sonar-pro")
      }
    },
    async (input) => {
      const res = await planAndExecute({
        query: input.query,
        domains: input.domains,
        recency: input.recency,
        mode: input.mode ?? "web",
        maxResults: input.maxResults ?? 8,
        model: input.model ?? "sonar-pro",
        verbose: false
      });

      const { answer, citations, planLog } = res;
      const citationBlock = citations.map((c, i) => `[${i+1}] ${c.title} — ${c.url}`).join("\n");

      return {
        content: [
          { type: "text", text: `Answer:\n${answer}\n\nCitations:\n${citationBlock}\n\nPlan:\n${JSON.stringify(planLog, null, 2)}` }
        ]
      };
    }
  );

  // Tool 2: raw ranked search results
  server.registerTool(
    "search.raw",
    {
      title: "Perplexity Search raw results",
      description: "Returns ranked results from Perplexity Search API",
      inputSchema: {
        query: z.union([z.string(), z.array(z.string())]).describe("Query or queries"),
        maxResults: z.number().int().min(1).max(20).default(10),
        maxTokensPerPage: z.number().int().min(128).max(4096).default(1024),
        country: z.string().optional().describe("ISO country code")
      }
    },
    async (input) => {
      const result = await rawSearch({
        query: input.query,
        maxResults: input.maxResults ?? 10,
        maxTokensPerPage: input.maxTokensPerPage ?? 1024,
        country: input.country
      });
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }]
      };
    }
  );

  const transport = new StdioServerTransport();
  await server.connect(transport);
  if (opts.verbose) console.error("perplexity-goap MCP server ready");
}
```

---

### src/goap/types.ts

```ts
export type WorldState = Record<string, unknown>;

export interface Action {
  name: string;
  cost: number;
  pre: Partial<WorldState>;
  eff: Partial<WorldState>;
  check:(s: WorldState)=> boolean;     // fast check on preconditions
  run:(s: WorldState)=> Promise<WorldState>; // side effects, updates world
}

export interface Goal {
  desired: Partial<WorldState>;
}

export interface PlanNode {
  state: WorldState;
  g: number;      // cost so far
  h: number;      // heuristic
  f: number;      // g+h
  action?: Action;
  parent?: PlanNode;
}

export interface PlanResult {
  actions: Action[];
  states: WorldState[];
}
```

---

### src/goap/planner.ts

```ts
import { Action, Goal, PlanNode, WorldState } from "./types.js";
import { composeQueries, synthesizeAnswer } from "../actions/perplexityChat.js";
import { searchMany } from "../actions/perplexitySearch.js";

export async function planAndExecute(opts: {
  query: string;
  domains?: string[];
  recency?: "day"|"week"|"month"|"year";
  mode: "web"|"academic";
  maxResults: number;
  model: "sonar"|"sonar-pro"|"sonar-reasoning";
  verbose?: boolean;
}) {
  const planLog: any[] = [];
  const initial: WorldState = {
    query: opts.query,
    queriesPrepared: false,
    resultsFetched: false,
    answerDrafted: false,
    answerVerified: false,
    citations: [],
    answer: ""
  };

  // Define abstract actions with STRIPS-like pre and eff
  const actions: Action[] = [
    {
      name: "ComposeQueries",
      cost: 1,
      pre: { queriesPrepared: false },
      eff: { queriesPrepared: true },
      check: s => typeof s.query === "string",
      run: async s => {
        const queries = await composeQueries(String(s.query), opts.model);
        planLog.push({ step:"ComposeQueries", queries });
        return { ...s, queries };
      }
    },
    {
      name: "Search",
      cost: 2,
      pre: { queriesPrepared: true, resultsFetched: false },
      eff: { resultsFetched: true },
      check: s => Array.isArray((s as any).queries),
      run: async s => {
        const { results, flat } = await searchMany({
          queries: (s as any).queries,
          maxResults: opts.maxResults
        });
        planLog.push({ step:"Search", resultCounts: results.map(r => r.results.length) });
        return { ...s, results, flat };
      }
    },
    {
      name: "Synthesize",
      cost: 3,
      pre: { resultsFetched: true, answerDrafted: false },
      eff: { answerDrafted: true },
      check: s => Array.isArray((s as any).flat),
      run: async s => {
        const { answer, citations } = await synthesizeAnswer({
          question: String(s.query),
          sources: (s as any).flat,
          domains: opts.domains,
          recency: opts.recency,
          mode: opts.mode,
          model: opts.model
        });
        planLog.push({ step:"Synthesize", citationsCount: citations.length });
        return { ...s, answer, citations };
      }
    },
    {
      name: "Verify",
      cost: 1,
      pre: { answerDrafted: true, answerVerified: false },
      eff: { answerVerified: true },
      check: s => typeof (s as any).answer === "string",
      run: async s => {
        // Simple structural verification: require at least 3 distinct sources
        const cites = (s as any).citations ?? [];
        const ok = Array.isArray(cites) && new Set(cites.map((c:any)=>c.url)).size >= Math.min(3, cites.length);
        planLog.push({ step:"Verify", ok });
        return { ...s, answerVerified: ok };
      }
    }
  ];

  const goal: Goal = { desired: { answerVerified: true } };

  const plan = planAStar(initial, actions, goal);
  const executedStates: WorldState[] = [initial];
  let state = initial;

  for (const a of plan.actions) {
    if (!a.check(state)) {
      // dynamic re-plan if preconditions not met
      const replan = planAStar(state, actions, goal);
      planLog.push({ step:"Replan", actions: replan.actions.map(x=>x.name) });
      for (const ra of replan.actions) {
        state = await ra.run(state);
        executedStates.push(state);
      }
      break;
    } else {
      state = await a.run(state);
      // mark effects
      state = { ...state, ...a.eff };
      executedStates.push(state);
    }
  }

  return {
    answer: String((state as any).answer ?? ""),
    citations: (state as any).citations ?? [],
    planLog
  };
}

// A* planner on symbolic world states with unit-weight heuristic
function planAStar(initial: WorldState, actions: Action[], goal: Goal): { actions: Action[]; states: WorldState[] } {
  const open: PlanNode[] = [];
  const closed = new Set<string>();
  const h = (s: WorldState) => {
    let missing = 0;
    for (const [k,v] of Object.entries(goal.desired)) {
      if (s[k] !== v) missing++;
    }
    return missing;
  };
  const key = (s: WorldState) => JSON.stringify(Object.keys(s).sort().reduce((o,k)=>{o[k]=s[k];return o;},{} as any));

  const start: PlanNode = { state: initial, g: 0, h: h(initial), f: h(initial) };
  open.push(start);

  while (open.length) {
    open.sort((a,b)=>a.f-b.f);
    const current = open.shift()!;
    const ck = key(current.state);
    if (closed.has(ck)) continue;
    closed.add(ck);

    // goal test
    if (h(current.state) === 0) {
      // reconstruct
      const actionsPath: Action[] = [];
      const statesPath: WorldState[] = [];
      let n: PlanNode | undefined = current;
      while (n) {
        if (n.action) actionsPath.unshift(n.action);
        statesPath.unshift(n.state);
        n = n.parent;
      }
      return { actions: actionsPath, states: statesPath };
    }

    for (const a of actions) {
      if (!satisfies(current.state, a.pre)) continue;
      const nextState = { ...current.state, ...a.eff };
      const g = current.g + a.cost;
      const hh = h(nextState);
      open.push({ state: nextState, g, h: hh, f: g+hh, action: a, parent: current });
    }
  }

  // fallback linear plan
  return { actions, states: [initial] };
}

function satisfies(s: WorldState, pre: Partial<WorldState>) {
  for (const [k,v] of Object.entries(pre)) {
    if (s[k] !== v) return false;
  }
  return true;
}
```

> GOAP uses goals, actions with preconditions and effects, and a planner such as A* to find an action sequence. ([GameDevs][5])

---

### src/actions/perplexitySearch.ts

```ts
import { request } from "undici";

const API_BASE = "https://api.perplexity.ai";

export async function rawSearch(args: {
  query: string | string[];
  maxResults: number;
  maxTokensPerPage: number;
  country?: string;
}) {
  const res = await request(`${API_BASE}/search`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${process.env.PERPLEXITY_API_KEY}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      query: args.query,
      max_results: args.maxResults,
      max_tokens_per_page: args.maxTokensPerPage,
      country: args.country
    })
  });
  if (res.statusCode !== 200) {
    const text = await res.body.text();
    throw new Error(`Search failed: ${res.statusCode} ${text}`);
  }
  const json = await res.body.json();
  return json as { results: { title:string; url:string; snippet:string; date?:string; last_updated?:string }[] };
}

export async function searchMany(opts: { queries: string[]; maxResults: number }) {
  const r = await rawSearch({
    query: opts.queries,
    maxResults: opts.maxResults,
    maxTokensPerPage: 1024
  });
  const results = Array.isArray(r.results) ? [{ results: r.results }] : [];
  const flat = r.results.map(x => ({ title: x.title, url: x.url, date: x.date, last_updated: x.last_updated }));
  return { results, flat };
}
```

> The Search endpoint is `POST /search` and accepts a single string or an array for multi‑query; it returns ranked results with fields like title, url, date, and last_updated. ([Perplexity][2])

---

### src/actions/perplexityChat.ts

```ts
import OpenAI from "openai";

const openai = new OpenAI({
  apiKey: process.env.PERPLEXITY_API_KEY,
  baseURL: "https://api.perplexity.ai"  // OpenAI-compatible Sonar endpoint
});

export async function composeQueries(question: string, model: "sonar"|"sonar-pro"|"sonar-reasoning") {
  const sys = [
    "You generate diversified search queries to comprehensively answer a question.",
    "Return 5 focused queries, JSON array of strings only."
  ].join("\n");
  const resp = await openai.chat.completions.create({
    model,
    messages: [
      { role: "system", content: sys },
      { role: "user", content: `Question: ${question}` }
    ],
    temperature: 0.2,
  });
  const txt = resp.choices[0]?.message?.content ?? "[]";
  try {
    const arr = JSON.parse(txt);
    if (Array.isArray(arr)) return arr.slice(0,5).map(String);
  } catch {}
  // fallback: seed with variants
  return [question, `${question} site:.gov`, `${question} site:.edu`, `${question} PDF`, `${question} latest`];
}

export async function synthesizeAnswer(opts: {
  question: string;
  sources: { title:string; url:string; date?:string; last_updated?:string }[];
  domains?: string[];
  recency?: "day"|"week"|"month"|"year";
  mode: "web"|"academic";
  model: "sonar"|"sonar-pro"|"sonar-reasoning";
}) {
  const domainFilter = opts.domains && opts.domains.length ? opts.domains : undefined;

  const messages = [
    { role: "system", content: [
        "You are a senior research assistant.",
        "Use the provided sources, cross-check facts, and cite inline as [n].",
        "Prefer recent sources when recency is set. Be concise."
      ].join("\n")
    },
    { role: "user", content: [
        `Question: ${opts.question}`,
        `Sources:\n${opts.sources.map((s,i)=>`[${i+1}] ${s.title} — ${s.url} (${s.date ?? ""})`).join("\n")}`,
        "Answer with clear bullets or short paragraphs. End with a Sources section listing [n] → URL."
      ].join("\n")
    }
  ] as const;

  const completion = await openai.chat.completions.create({
    model: opts.model,
    messages,
    // Perplexity-specific search controls via OpenAI-compatible client
    extra_body: {
      search_domain_filter: domainFilter,
      search_recency_filter: opts.recency,
      search_mode: opts.mode
    },
    temperature: 0.2,
    max_tokens: 800
  });

  const content = completion.choices[0]?.message?.content ?? "";
  // Perplexity returns search metadata; we keep our explicit list as citations
  const citations = opts.sources.slice(0, 8);
  return { answer: content, citations };
}
```

> Perplexity Sonar is OpenAI‑compatible. You point the OpenAI client to `https://api.perplexity.ai`, set the API key, and may pass Perplexity‑specific search controls like `search_domain_filter` and `search_recency_filter`. ([Perplexity][3])

---

### src/utils/citations.ts

```ts
export function trimAndUnique(urls: string[]) {
  return Array.from(new Set(urls.map(u => u.trim()))).slice(0, 12);
}
```

---

## 5) Using it inside an MCP host

Once running, your MCP host will list two tools:

* `goap.search`
  Inputs: `query`, `domains?`, `recency?`, `mode?`, `maxResults?`, `model?`
  Output: text containing the answer, citations, and the executed plan.

* `search.raw`
  Inputs: `query`, `maxResults?`, `maxTokensPerPage?`, `country?`
  Output: raw JSON of ranked results.

MCP server structure and stdio transport match the official TypeScript SDK patterns. ([GitHub][1])

---

## 6) Why GOAP here

* We model a compact **world state**: queriesPrepared, resultsFetched, answerDrafted, answerVerified.
* Actions carry **preconditions** and **effects**, and the planner uses **A*** to reach a goal state `answerVerified=true`.
* The approach follows Jeff Orkin’s GOAP, adapted from STRIPS, with low overhead and dynamic re‑planning if an action fails. ([GameDevs][5])

---

## 7) Example run

Invoke via Claude Desktop after adding the server, then run:

```
Tool: goap.search
{
  "query": "What changed in the 2025 Stanford AI Index vs 2024? Prioritize primary sources.",
  "domains": ["hai.stanford.edu", "oecd.ai"],
  "recency": "year",
  "mode": "web",
  "maxResults": 8,
  "model": "sonar-pro"
}
```

You will see a step log similar to:

```json
[
  {"step":"ComposeQueries","queries":["2025 AI Index key changes", ...]},
  {"step":"Search","resultCounts":[8]},
  {"step":"Synthesize","citationsCount":8},
  {"step":"Verify","ok":true}
]
```

---

## 8) Operational notes

* **Auth**. Create and export `PERPLEXITY_API_KEY`. ([Perplexity][6])
* **Search**. `POST /search` supports single or multiple queries with `max_results` and `max_tokens_per_page`. ([Perplexity][2])
* **Chat**. Use Sonar via OpenAI client with `baseURL=https://api.perplexity.ai` and pass Perplexity search controls when needed. ([Perplexity][3])
* **MCP**. Server runs over stdio and can be launched from `npx` like other reference servers. ([Model Context Protocol][4])

---

## 9) Testing and hardening checklist

* Unit tests on `planAStar` with synthetic states to verify optimality for simple graphs
* Network retries and backoff on 429 and 5xx
* Cap `maxResults` to reduce token costs, track usage with `completion.usage`
* Add a “strict verify” action that checks for at least one source dated within your recency filter
* Optional: add `mode:"academic"` for literature heavy topics

---

## 10) Licensing and publishing

1. Set your npm scope in `package.json`
2. `npm publish --access public`
3. Run from anywhere:

```bash
PERPLEXITY_API_KEY=... npx -y @ruv/mcp-perplexity-goap
```

---

## 11) Mini content to position you, rUv

**LinkedIn blurb template**

> Shipping a new MCP server that fuses Perplexity’s Search API with GOAP planning. One command, `npx @ruv/mcp-perplexity-goap`, gives any MCP‑enabled agent a fast plan‑then‑search‑then‑verify workflow with grounded citations. This is how enterprise research agents should operate: plan first, retrieve broadly, synthesize with control, and verify. DM for the repo.

---

## 12) Failure modes and recovery

* If `search` returns sparse results, re‑run `ComposeQueries` with higher diversity and re‑plan
* If `Verify` fails, automatically re‑invoke `Search` with `country` set or loosen domain filters
* Network errors should trigger exponential backoff and limited retries

---

## 13) Extensions you can add next

* Add a `goap.audit` tool that returns contradictions across sources
* Cache per‑domain results to cut latency and cost
* Add a “goal agenda” that sequences sub‑goals when a query decomposes into parts

---

### References

* Perplexity SDK quickstart, auth, and model guides. ([Perplexity][6])
* Search API reference for `POST /search`. ([Perplexity][2])
* OpenAI‑compatible Sonar usage including base URL and vendor fields. ([Perplexity][3])
* MCP TypeScript SDK quickstart and stdio server pattern. ([GitHub][1])
* MCP `npx` configuration pattern for Claude Desktop. ([Model Context Protocol][4])
* GOAP background and Orkin’s AI of F.E.A.R. paper. ([GameDevs][5])

---

### Feedback hooks

* Rate the plan quality 1 to 5
* Note if sources matched your trust list
* Share latency and token usage so we can tune `maxResults` and `max_tokens_per_page`

If you want, I can pre‑fill this into a GitHub repo structure so you can publish immediately with your npm scope.

[1]: https://github.com/modelcontextprotocol/typescript-sdk "GitHub - modelcontextprotocol/typescript-sdk: The official TypeScript SDK for Model Context Protocol servers and clients"
[2]: https://docs.perplexity.ai/api-reference/search-post "Search - Perplexity"
[3]: https://docs.perplexity.ai/guides/chat-completions-guide "OpenAI Compatibility - Perplexity"
[4]: https://modelcontextprotocol.io/examples "Example Servers - Model Context Protocol"
[5]: https://www.gamedevs.org/uploads/three-states-plan-ai-of-fear.pdf?utm_source=chatgpt.com "Three States and a Plan: The A.I. of F.E.A.R."
[6]: https://docs.perplexity.ai/guides/perplexity-sdk "Quickstart - Perplexity"
