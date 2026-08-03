/**
 * Deterministic projection + digest of a compiled guidance bundle.
 *
 * The compiler stamps every rule with createdAt/updatedAt (Date.now()) and the
 * manifest with compiledAt, so the raw bundle is NOT stable across runs. We
 * digest only the semantic fields — rule id, text, intents, domains, risk,
 * priority, scope — so the same GUIDANCE.md always yields the same hash.
 */
import { hashJson } from './hash.mjs';

const projRule = (r) => ({
  id: r.id,
  text: r.text,
  intents: [...(r.intents ?? [])].map(String).sort(),
  domains: [...(r.domains ?? [])].map(String).sort(),
  toolClasses: [...(r.toolClasses ?? [])].map(String).sort(),
  repoScopes: [...(r.repoScopes ?? [])].map(String).sort(),
  riskClass: r.riskClass,
  priority: r.priority,
  isConstitution: !!r.isConstitution,
});

export function projectBundle(bundle) {
  return {
    constitution: (bundle.constitution?.rules ?? []).map(projRule),
    shards: (bundle.shards ?? []).map((s) => projRule(s.rule)),
    counts: {
      constitutionRules: bundle.manifest?.constitutionRules ?? 0,
      shardRules: bundle.manifest?.shardRules ?? 0,
      totalRules: bundle.manifest?.totalRules ?? 0,
    },
  };
}

export const digestBundle = (bundle) => hashJson(projectBundle(bundle));
