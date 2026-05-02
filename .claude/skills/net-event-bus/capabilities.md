# Capabilities — Capability-Based Routing

Read this when the user wants to **place compute on the right node**, not push events through a topic. The pattern: each node declares what it can do (GPU model, loaded LLMs, callable tools, free-form tags), other nodes query the mesh by filter, and you get back a list of node ids you can talk to directly. Killed by the routed-handshake path so it works across NAT.

This is the primitive Kafka / NATS / Redis Streams / Pulsar do not have. There is no broker subject like `gpu-work` you subscribe to and hope something with a GPU is listening; you ask the mesh "who has an NVIDIA GPU with ≥ 16 GB VRAM and `llama-3.1-70b` already loaded?" and unicast to the winner.

If the user wants topic-style fan-out, use `apis.md` (named channels) or `patterns.md` § "I want a consumer that subscribes to a topic". This file is for placement.

---

## The capability model

Each node builds a `CapabilitySet` and announces it. Verbatim from `net/crates/net/src/adapter/net/behavior/capability.rs:798`:

```rust
pub struct CapabilitySet {
    pub hardware: HardwareCapabilities,
    pub software: SoftwareCapabilities,
    pub models: Vec<ModelCapability>,
    pub tools: Vec<ToolCapability>,
    pub tags: Vec<String>,
    pub limits: ResourceLimits,
}
```

Field names you'll touch (verbatim from `capability.rs`):

- `HardwareCapabilities` (`:192`): `cpu_cores`, `cpu_threads`, `memory_mb`, `gpu: Option<GpuInfo>`, `additional_gpus`, `storage_mb`, `network_mbps`, `accelerators`.
- `GpuInfo` (`:60`): `vendor: GpuVendor`, `model`, `vram_mb`, `compute_units`, `tensor_cores`, `fp16_tflops_x10`.
- `ModelCapability` (`:387`): `model_id`, `family`, `parameters_b_x10`, `context_length`, `quantization`, `modalities: Vec<Modality>`, `tokens_per_sec`, `loaded`.
- `ToolCapability` (`:471`): `tool_id`, `name`, `version`, `input_schema`, `output_schema`, `requires`, `estimated_time_ms`, `stateless`.
- `ResourceLimits` (`:548`): `max_concurrent_requests`, `max_tokens_per_request`, `rate_limit_rpm`, `max_batch_size`, `max_input_bytes`, `max_output_bytes`.

`GpuVendor` is `Nvidia | Amd | Intel | Apple | Qualcomm | Unknown`. `Modality` is `Text | Image | Audio | Video | Code | Embedding | ToolUse`. `parameters_b_x10` and `fp16_tflops_x10` are integer-encoded (× 10) to dodge float precision loss on the wire. `tags` is the free-form escape hatch (`prod`, `customer:acme`, `gpu-pool-a`).

---

## Announcing

`announce_capabilities(caps)` pushes the set to every directly-connected peer over subprotocol `0x0C00` (`net/crates/net/src/adapter/net/behavior/broadcast.rs:12`). The announcer self-indexes too, so single-node tests round-trip. Multi-hop propagation is deferred — peers more than one hop away will not see the announcement today.

Re-announce to update. Subsequent calls go through the `CapabilityDiff` machinery in `net/crates/net/src/adapter/net/behavior/diff.rs` — incremental, signed — so steady-state changes (a model gets loaded, a tag toggles) cost ~50 bytes on the wire instead of a full re-broadcast. You don't drive this directly; the mesh figures it out from your last announced set.

Default TTL is 5 minutes. Override with `announce_capabilities_with(caps, ttl, sign)` (Rust). Re-announce before TTL elapses or peers will GC the entry.

---

## Querying — three shapes

| Shape | Returns | When to use |
|---|---|---|
| `find_nodes(filter)` | every node id whose latest announcement matches | broadcast a request to all candidates, or hand-pick |
| `find_best_node(req)` | a single node id, the highest-scoring match | "give me one node, optimize for these weights" — the workload-scheduler pattern |
| `find_nodes_scoped(filter, scope)` | matches narrowed to a tenant / region / subnet / global pool | multi-tenant or multi-region deployments |

`find_best_node` ranks via `CapabilityRequirement` (`capability.rs:1367`):

```rust
pub struct CapabilityRequirement {
    pub filter: CapabilityFilter,
    pub prefer_more_memory: f32,
    pub prefer_more_vram: f32,
    pub prefer_faster_inference: f32,
    pub prefer_loaded_models: f32,
}
```

Weights are clamped to `[0.0, 1.0]`. `prefer_loaded_models = 1.0` is the right setting for "don't pay cold-start latency" — a node that already has the model loaded wins over an idle GPU with more VRAM.

`CapabilityFilter` (`capability.rs:1207`) — what every shape filters by:

```rust
pub struct CapabilityFilter {
    pub require_tags: Vec<String>,
    pub require_models: Vec<String>,
    pub require_tools: Vec<String>,
    pub min_memory_mb: Option<u32>,
    pub require_gpu: bool,
    pub gpu_vendor: Option<GpuVendor>,
    pub min_vram_mb: Option<u32>,
    pub min_context_length: Option<u32>,
    pub require_modalities: Vec<Modality>,
}
```

`require_tags` is AND (every tag must be present). `require_models` and `require_tools` are OR (any one match satisfies).

---

## Per-SDK examples

### Rust

```rust
use net_sdk::capabilities::{
    CapabilityFilter, CapabilityRequirement, CapabilitySet, GpuInfo, GpuVendor,
    HardwareCapabilities,
};
use net_sdk::mesh::MeshBuilder;

let node = MeshBuilder::new("0.0.0.0:0", &psk).build().await?;
let hw = HardwareCapabilities::new()
    .with_cpu(16, 32)
    .with_memory(65_536)
    .with_gpu(GpuInfo::new(GpuVendor::Nvidia, "RTX 4090", 24_576));
node.announce_capabilities(CapabilitySet::new().with_hardware(hw).add_tag("prod")).await?;

let req = CapabilityRequirement::from_filter(
    CapabilityFilter::new().with_gpu_vendor(GpuVendor::Nvidia).with_min_vram(16_384),
);
if let Some(node_id) = node.find_best_node(&req) { /* unicast to node_id */ }
```

**Key facts:**
- Public types live under `net_sdk::capabilities::*` (re-exports in `net/crates/net/sdk/src/capabilities.rs:62-67`).
- Mesh wrapper is `net_sdk::mesh::Mesh`, built via `MeshBuilder`. All five methods (`announce_capabilities`, `announce_capabilities_with`, `find_nodes`, `find_nodes_scoped`, `find_best_node`, `find_best_node_scoped`) live on it (`net/crates/net/sdk/src/mesh.rs:714-775`).
- `ScopeFilter<'a>` borrows its tenant / region strings — keep them alive across the call.

### TypeScript

```typescript
import { MeshNode, normalizeGpuVendor } from '@ai2070/net-sdk';

const node = await MeshNode.create({ bindAddr: '0.0.0.0:0', psk });
await node.announceCapabilities({
  hardware: {
    cpuCores: 16, memoryMb: 65_536,
    gpu: { vendor: 'nvidia', model: 'RTX 4090', vramMb: 24_576 },
  },
  tags: ['prod'],
  models: [{ modelId: 'llama-3.1-70b', family: 'llama', loaded: true }],
});
const peers: bigint[] = node.findNodes({
  requireGpu: true, gpuVendor: normalizeGpuVendor('NVIDIA'), minVramMb: 16_384,
});
```

**Key facts:**
- `MeshNode` exposes `announceCapabilities`, `findNodes`, `findNodesScoped` (`net/crates/net/sdk-ts/src/mesh.ts:528-566`). Types: `net/crates/net/sdk-ts/src/capabilities.ts`.
- **No `findBestNode` in the TS SDK or NAPI binding today** (`net/crates/net/bindings/node/src/capabilities.rs` exposes only the filter path). For best-match scoring in TS, use `findNodes` + score on the caller side, or drop to Rust / Go on the placement node.
- Node ids are `bigint` — routinely exceed `Number.MAX_SAFE_INTEGER`. Don't naively `JSON.stringify`.
- `ScopeFilter` is a tagged union: `{ kind: 'any' | 'globalOnly' | 'sameSubnet' | 'tenant' | 'tenants' | 'region' | 'regions', ... }`.

### Python

```python
from net import NetMesh, normalize_gpu_vendor  # PyO3 native module

node = NetMesh("0.0.0.0:0", psk_hex)
node.start()
node.announce_capabilities({
    "hardware": {"cpu_cores": 16, "memory_mb": 65_536,
        "gpu": {"vendor": "nvidia", "model": "RTX 4090", "vram_mb": 24_576}},
    "tags": ["prod"],
    "models": [{"model_id": "llama-3.1-70b", "family": "llama", "loaded": True}],
})
peers = node.find_nodes({
    "require_gpu": True, "gpu_vendor": normalize_gpu_vendor("NVIDIA"), "min_vram_mb": 16_384,
})
```

**Key facts:**
- Capability surface lives on the **native** `_net.NetMesh` PyO3 class — `from net import NetMesh`. The `net_sdk.MeshNode` wrapper does not re-export these methods today; reach through `mesh._native` if you're already holding a wrapper.
- POJO shape is `dict`s with `snake_case` keys (mirrors the C/Go JSON contract). Source: `net/crates/net/bindings/python/src/capabilities.rs`.
- **No `find_best_node` in the PyO3 binding today** (parallel gap with NAPI). Use `find_nodes` and pick on the caller side, or drop to Rust / Go for scoring.
- Scope dict `kind` accepts both `snake_case` and `camelCase`: `same_subnet` and `sameSubnet` both work (`bindings/python/src/capabilities.rs:454-457`).

### Go and C

Both have the **full** surface (announce + find_nodes + find_nodes_scoped + find_best_node + find_best_node_scoped). Headers / signatures:

- **Go:** `go/capabilities.go` — `MeshNode.AnnounceCapabilities`, `FindNodes`, `FindNodesScoped`, `FindBestNode`, `FindBestNodeScoped`. `FindBestNode` returns `(uint64, bool, error)` — the bool disambiguates "no match" from `nodeId == 0` (a valid id).
- **C:** `go/net.h` — `net_mesh_announce_capabilities`, `net_mesh_find_nodes`, `net_mesh_find_nodes_scoped`, `net_mesh_find_best_node`, `net_mesh_find_best_node_scoped`. JSON in / JSON out; free returned strings via `net_free_string`.

Both also expose `net_normalize_gpu_vendor` / `NormalizeGpuVendor` (see § GPU vendor normalization below).

---

## Worked example: GPU inference routing

The full pattern — GPU node announces; requester queries; requester unicasts work.

```rust
// GPU host
use net_sdk::capabilities::{
    CapabilitySet, GpuInfo, GpuVendor, HardwareCapabilities, ModelCapability, Modality,
};
use net_sdk::mesh::MeshBuilder;

let gpu_node = MeshBuilder::new("0.0.0.0:0", &psk).build().await?;
gpu_node.announce_capabilities(
    CapabilitySet::new()
        .with_hardware(
            HardwareCapabilities::new()
                .with_memory(131_072)
                .with_gpu(GpuInfo::new(GpuVendor::Nvidia, "H100", 81_920)),
        )
        .add_model(
            ModelCapability::new("llama-3.1-70b", "llama")
                .with_context_length(131_072)
                .add_modality(Modality::Text)
                .with_loaded(true),
        )
        .add_tag("prod"),
).await?;

// Requester
use net_sdk::capabilities::{CapabilityFilter, CapabilityRequirement};

let req = CapabilityRequirement {
    filter: CapabilityFilter::new()
        .with_gpu_vendor(GpuVendor::Nvidia)
        .with_min_vram(16_384)
        .require_model("llama-3.1-70b")
        .require_tag("prod"),
    prefer_loaded_models: 1.0,
    prefer_more_vram: 0.3,
    ..Default::default()
};
let target = requester.find_best_node(&req).ok_or("no GPU peer matched")?;
// Open a stream / channel to `target` and send the prompt.
```

This is **Net used as a workload scheduler, not a broker.** The publisher's roster machinery never enters the picture; you picked the destination yourself by capability. Subprotocol traffic rides the same encrypted Noise session as any other mesh packet — relays forward bytes they cannot read.

---

## Scopes — capping blast radius

When you have many tenants or regions, you don't want a `find_nodes` query to span everyone. `find_nodes_scoped(filter, scope)` filters candidates through their `scope:*` reserved tags before returning.

Variants (Rust: `capability.rs:702-725`, TS: `sdk-ts/src/capabilities.ts:391-398`):

- `Any` — every peer except those tagged `scope:subnet-local` (which always require `SameSubnet`).
- `GlobalOnly` — only peers with no `scope:*` tag at all.
- `SameSubnet` — peers in the caller's subnet (caller-supplied predicate).
- `Tenant("acme")` — that tenant **plus** untagged Global peers.
- `Tenants(["acme", "beta"])` — any of those tenants **plus** Global.
- `Region("us-west")` / `Regions([...])` — same shape, region tags.

The "**plus** Global" is deliberate: untagged peers stay discoverable so a node that doesn't tag itself doesn't fall off the map by accident. To exclude untagged nodes, use `GlobalOnly` (its inverse — only untagged) or filter the result.

**Empty arrays / strings fall through to `Any`.** `Tenants([])`, `Regions([])`, `Tenant("")`, `Region("")` all resolve to `Any` rather than "match nothing" — empty scope ids would otherwise pin the query to Global-only candidates, which is rarely what the caller meant. Consistent across all SDKs.

To advertise as scoped, add the reserved tag on the announcing side. Rust: `CapabilitySet::with_tenant_scope("acme")` → `scope:tenant:acme`; `with_region_scope("us-west")` → `scope:region:us-west`; `with_subnet_local_scope()` → `scope:subnet-local`. Strictest scope wins: a peer tagged both `scope:subnet-local` and `scope:tenant:acme` resolves to `SubnetLocal`.

`scope:*` is a **discovery filter, not a routing gate.** Wire format and forwarders are unchanged. To deny packets across boundaries, see `concepts.md` § Subnets.

---

## GPU vendor normalization

```typescript
import { normalizeGpuVendor } from '@ai2070/net-sdk';
normalizeGpuVendor('NVIDIA');   // 'nvidia'
normalizeGpuVendor('  AMD  ');  // 'amd' (parser is to-lower; spacing handled at filter-build time)
normalizeGpuVendor('rocm');     // 'unknown'
```

Same helper in every SDK:
- Rust: `GpuVendor::from(...)` plus `parse_gpu_vendor` in the binding source (or just use the `GpuVendor` enum directly — it's already canonical).
- Python: `from net import normalize_gpu_vendor`.
- Go: `net.NormalizeGpuVendor(s string) string`.
- C: `net_normalize_gpu_vendor(...)`.

Canonical lowercase values: `nvidia | amd | intel | apple | qualcomm | unknown`. Use this **before** building a filter so a misspelled vendor string doesn't silently collapse to `unknown` and match nothing. The on-wire `GpuVendor` enum tag is what's actually compared inside the index — the string is just the JSON-side spelling.

---

## What this is NOT

Capability routing is for **placement** (pick a node), not **delivery** (the bus still does fan-out).

- Don't build a topic system on top of `find_nodes`. Re-running the query on every emit is wasteful and the latest-announcement-wins index gives you no ordering guarantees.
- If you need ordered durable delivery to the node you picked, layer it: capability-route to find the `node_id`, then either
  - publish on a channel that node has subscribed to (`apis.md` § Named channels), or
  - open a per-peer reliable stream (`Reliability::Reliable`) and send directly.
- The capability index is **per-node, eventually consistent**. Two nodes querying at the same instant may see different candidate sets if an announcement is in flight. Don't treat results as global truth.
- Multi-hop propagation is deferred today. If your mesh has a relay-only node between announcer and querier, the querier won't see the announcement until the announcer connects directly. Plan capacity around direct-peer topology.

---

## Cross-references

- `apis.md` — general SDK shape, channel/firehose surfaces, transport selection.
- `mesh.md` — bind addresses, PSK / identity, peer setup. You need a live mesh node before any of this works.
- `concepts.md` § Subnets — how subnet policies (a *routing* gate) differ from `scope:*` tags (a *discovery* filter).
- `patterns.md` § "I want per-tenant capability discovery without standing up subnets" — the recipe-level summary that links here.
