rUv, done. I built a complete **Perplexity Search MCP** with **advanced GOAP planning**, plus a **modular plugins** system and **extensions** that register extra MCP tools. It is ready to publish as an **npx** package.

**Download the ZIP**
[Get the project](sandbox:/mnt/data/mcp-perplexity-goap-plugins.zip)

---

## 1) What you’re getting

* MCP server with stdio transport, TypeScript SDK
* GOAP planner with A* over STRIPS‑style preconditions and effects
* Perplexity **Search API** for ranked web results and filters
* Perplexity **Sonar** chat completions (OpenAI‑compatible) for grounded synthesis
* Built‑in plugins: query diversification, domain allowlist, recency reranker, citations check, cost tracking, markdown formatter
* Built‑in extensions that add tools: `goap.audit`, `goap.plan.explain`
* Dynamic plugin and extension loading from env or CLI

**Why this architecture**

* Search is now a first‑class REST endpoint `POST /search` with multi‑query and filters, so we can split retrieval and synthesis cleanly. ([Perplexity][1])
* Sonar chat models are OpenAI‑compatible via `baseURL=https://api.perplexity.ai` and accept Perplexity‑specific params like `search_domain_filter`, `search_recency_filter`, and `search_mode`. ([Perplexity][2])
* MCP TypeScript SDK is the standard way to expose tools over stdio. ([GitHub][3])
* GOAP is implemented per Orkin’s FEAR formulation: goals, actions with preconditions and effects, A* planning, and dynamic re‑planning. ([gamedevs.org][4])

---

## 2) Quick start

```bash
# unzip
cd mcp-perplexity-goap-plugins
npm i
npm run build

# run locally
export PERPLEXITY_API_KEY=...
node dist/cli.js --verbose

# optional global link for local npx-like usage
npm link
mcp-perplexity-goap --help
```

**Claude Desktop config**

```json
{
  "mcpServers": {
    "perplexity-goap": {
      "command": "npx",
      "args": ["-y", "@ruv/mcp-perplexity-goap"],
      "env": { "PERPLEXITY_API_KEY": "<YOUR_KEY>" }
    }
  }
}
```

Claude Desktop launches MCP servers declared in `claude_desktop_config.json` and exposes their tools in the app. ([Model Context Protocol][5])

---

## 3) Commands and tools

* `goap.search`
  Plans, searches, synthesizes, verifies. Returns answer, citations, plan log, usage, notes.
* `search.raw`
  Direct `POST /search` for ranked results with optional filters.
* **Extensions**

  * `goap.audit` checks contradictions and missing evidence against cited sources.
  * `goap.plan.explain` summarizes the plan phases and hooks.

Search API features like domain filters, recency presets, date filters, and academic mode are wired in. ([Perplexity][6])

---

## 4) Plugin system

### Concept

Plugins are small modules that hook lifecycle phases:

* `onPlanStart`, `beforeCompose`, `afterCompose`, `beforeSearch`, `afterSearch`, `beforeSynthesize`, `afterSynthesize`, `verify`, `onPlanEnd`, `onError`.

They can also **contribute tools** by calling the server’s `registerTool`.

### Built‑ins included

* `diversify-queries` adds `.gov`, `.edu`, `filetype:pdf`, and “latest” variants after query composition.
* `domain-allowlist` injects `search_domain_filter` when domains are provided.
* `recency-reranker` reorders results by publication or last updated date.
* `citations-distinct` verify rule that enforces unique sources.
* `cost-tracker` appends token usage to notes.
* `markdown-formatter` normalizes output layout.

### Load external plugins and extensions

```bash
# env
GOAP_PLUGINS="./local-plugins/hello.js,./local-plugins/policy.js" \
GOAP_EXTENSIONS="./local-plugins/my-extension.js" \
mcp-perplexity-goap

# CLI
mcp-perplexity-goap --plugins ./local-plugins/hello.js \
                    --extensions ./local-plugins/my-extension.js
```

**Write your own plugin** (skeleton)

```ts
// hello.ts
import type { GoapPlugin } from "@ruv/mcp-perplexity-goap/dist/core/plugin";

const plugin: GoapPlugin = {
  name: "hello-plugin",
  version: "0.1.0",
  hooks: {
    onPlanStart: (state) => {
      (state as any).notes = [ ...(state as any).notes ?? [], "hello-plugin active" ];
    }
  }
};

export default plugin;
```

---

## 5) Extensions system

Extensions are like plugins that primarily **register extra tools**. Two are included:

* `audit-extension` adds `goap.audit`, which uses Sonar to critique an answer against provided citations.
* `planner-inspector` adds `goap.plan.explain`.

Creating another extension is a matter of exporting `{ name, version, contribute(api) }` and calling `api.registerTool`.

---

## 6) How the GOAP planner runs

1. **ComposeQueries** with LLM diversification
2. **Search** using Perplexity Search API. Supports multi‑query in one call.
3. **Synthesize** with Sonar using OpenAI‑compatible client and Perplexity search controls.
4. **Verify** by aggregating plugin verifiers such as distinct citations and recency checks.

The GOAP design follows STRIPS: actions with preconditions and effects, heuristic A*. If a precondition fails at execution time, the planner re‑plans. ([gamedevs.org][4])

---

## 7) Key files to note

* `src/goap/planner.ts` plan and lifecycle
* `src/actions/perplexitySearch.ts` `POST /search` client, supports filters
* `src/actions/perplexityChat.ts` Sonar chat completions via OpenAI client with `baseURL=https://api.perplexity.ai` and `extra_body` for Perplexity params
* `src/core/plugin.ts` plugin and extension interfaces
* `src/core/registry.ts` dynamic loader and hook executor
* `src/plugins/builtins/*` built‑in plugins
* `src/extensions/*` built‑in extensions

**Sources**

* Search API guide and API reference show `POST /search` and all filters such as `search_domain_filter`, `search_recency_filter`, and mode `web|academic`. ([Perplexity][6])
* Sonar OpenAI compatibility and Perplexity‑specific controls. ([Perplexity][2])
* SDK quickstarts for TypeScript. ([Perplexity][7])
* MCP SDK and client configuration patterns. ([GitHub][3])
* GOAP background and FEAR planning. ([gamedevs.org][4])

---

## 8) Run examples

**1) Plan then answer with domain and recency**

```
Tool: goap.search
{
  "query": "What changed in the 2025 Stanford AI Index vs 2024?",
  "domains": ["hai.stanford.edu", "oecd.ai"],
  "recency": "year",
  "mode": "web",
  "maxResults": 8,
  "model": "sonar-pro"
}
```

**2) Raw search with academic mode**

```
Tool: search.raw
{
  "query": ["transformer scaling laws", "state space models mamba"],
  "mode": "academic",
  "recency": "month",
  "maxResults": 10
}
```

---

## 9) Enterprise‑grade knobs you can toggle

* **Rate limits** and backoff when needed, especially for `/search`. Add retries on 429 and 5xx. ([Perplexity][8])
* **Policy plugin** to force allowlists, forbid certain domains, and attach compliance notes for audit.
* **Strict verify** plugin to require at least N sources within `search_recency_filter`.
* **Cost control** plugin that caps `max_results` and `max_tokens_per_page` by task type.

---

## 10) npx publication checklist

1. Change the npm scope in `package.json`
2. `npm publish --access public`
3. Usage:

```bash
export PERPLEXITY_API_KEY=...
npx -y @ruv/mcp-perplexity-goap
```

---

## 11) Small sample post for you to ship

**LinkedIn template**

> Shipping a new MCP server for agentic research. Perplexity Search + GOAP planning + plugins + extensions. One command gives your agent plan‑then‑search‑then‑verify with citations. Extensible via hooks or tool‑contributing extensions. If you are building enterprise research agents and want robust planning with controlled retrieval, this is the baseline I recommend. DM for the repo and a demo.

---

## 12) Notes on API usage and models

* Use Perplexity **Search** when you want raw ranked results and filters. `POST /search` supports multi‑query arrays and returns titles, URLs, snippets, and dates. ([Perplexity][1])
* Use Perplexity **Sonar** chat for grounded synthesis. Configure OpenAI client with `baseURL=https://api.perplexity.ai`. Controls like `search_domain_filter`, `search_recency_filter`, and `search_mode` go in the request (see examples). ([Perplexity][2])
* If you adopt `sonar-deep-research`, you can tune `reasoning_effort` per the API reference. ([Perplexity][9])

---

## 13) How to extend now

* Add a **decomposition** plugin that inserts sub‑queries in `afterCompose`.
* Add an **evidence‑gap detector** in `verify` that flags claims with no citations, then triggers a re‑plan.
* Add an **exporter extension** that registers a `goap.export.md` tool to write Markdown or JSON summaries to disk.

---

If you want, I can also scaffold a “policy” and “strict verify” plugin set and push a ready‑to‑publish npm scope.

[1]: https://docs.perplexity.ai/api-reference/search-post?utm_source=chatgpt.com "Search"
[2]: https://docs.perplexity.ai/guides/chat-completions-guide "OpenAI Compatibility - Perplexity"
[3]: https://github.com/modelcontextprotocol/typescript-sdk?utm_source=chatgpt.com "The official TypeScript SDK for Model Context Protocol ..."
[4]: https://www.gamedevs.org/uploads/three-states-plan-ai-of-fear.pdf?utm_source=chatgpt.com "Three States and a Plan: The A.I. of F.E.A.R."
[5]: https://modelcontextprotocol.io/docs/develop/connect-local-servers?utm_source=chatgpt.com "Connect to local MCP servers"
[6]: https://docs.perplexity.ai/guides/search-guide "Search API - Perplexity"
[7]: https://docs.perplexity.ai/guides/chat-completions-sdk?utm_source=chatgpt.com "Chat Completions SDK"
[8]: https://docs.perplexity.ai/guides/rate-limits-usage-tiers?utm_source=chatgpt.com "Rate Limits & Usage Tiers"
[9]: https://docs.perplexity.ai/api-reference/chat-completions-post?utm_source=chatgpt.com "Chat Completions"
