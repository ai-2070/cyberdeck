# Subprotocol Registry

Formalizes the 16-bit `subprotocol_id` field in every Net header. Provides a registry for protocol handlers, version negotiation between peers, and an opaque forwarding guarantee for unknown protocols.

## Subprotocol IDs

Every Net packet carries a `subprotocol_id: u16` identifying how the payload should be interpreted. The ID space is partitioned:

| Range | Purpose |
|-------|---------|
| `0x0000` | Plain events (no subprotocol) |
| `0x0001..0x03FF` | Reserved for core |
| `0x0400` | Causal events |
| `0x0401` | State snapshots |
| `0x0500` | Daemon migration |
| `0x0600` | Subprotocol negotiation |
| `0x0700` | Continuity proofs |
| `0x0701` | Fork announcements |
| `0x0702` | Continuity proof transfer |
| `0x0800` | Partition detection |
| `0x0801` | Log reconciliation |
| `0x0900` | Replica group coordination (reserved) |
| `0x1000..0xEFFF` | Vendor / third-party |
| `0xF000..0xFFFF` | Experimental / ephemeral |

## Descriptors

Each registered subprotocol has a `SubprotocolDescriptor`:

```rust
pub struct SubprotocolDescriptor {
    pub id: u16,                              // Header field value
    pub name: String,                         // Human-readable (e.g., "causal")
    pub version: SubprotocolVersion,          // This handler's version
    pub min_compatible: SubprotocolVersion,   // Minimum peer version accepted
    pub handler_present: bool,                // false = opaque forwarding only
}

pub struct SubprotocolVersion {
    pub major: u8,   // Breaking changes
    pub minor: u8,   // Backward-compatible changes
}
```

Versions use 2-byte wire format (`[major, minor]`). Compatibility check: both peers' version must satisfy the other's `min_compatible`.

## Registry

`SubprotocolRegistry` maps IDs to descriptors and handlers.

```rust
impl SubprotocolRegistry {
    fn register(&self, descriptor: SubprotocolDescriptor) -> Result<(), RegistryError>
    fn lookup(&self, id: u16) -> Option<SubprotocolDescriptor>
    fn is_handled(&self, id: u16) -> bool
    fn all_descriptors(&self) -> Vec<SubprotocolDescriptor>
}
```

**Opaque forwarding guarantee:** Packets with unregistered `subprotocol_id` values are forwarded to the next hop without modification. This allows new protocols to be deployed incrementally -- intermediate nodes don't need to understand a protocol to forward it.

## Version Negotiation

When two peers establish a session, they exchange `SubprotocolManifest`s listing their supported subprotocols and versions.

```rust
pub struct SubprotocolManifest {
    pub entries: Vec<ManifestEntry>,
}

pub struct ManifestEntry {
    pub id: u16,
    pub version: SubprotocolVersion,
    pub min_compatible: SubprotocolVersion,
}
```

`NegotiatedSet` is the result of negotiation -- the subset of subprotocols both peers support at compatible versions. Packets using non-negotiated subprotocols fall back to opaque forwarding.

**Subprotocol ID for negotiation itself:** `0x0600`.

## Capability Advertisement

Nodes announce supported subprotocols through the capability graph. `SubprotocolRegistry::enrich_capabilities()` adds a tag `subprotocol:0x{id:04x}` for each handled subprotocol to the node's `CapabilitySet`. This enables capability-driven routing -- a migration request routes to the nearest node advertising `subprotocol:0x0500`.

```rust
// On node startup: enrich capabilities with subprotocol tags
let caps = subprotocol_registry.enrich_capabilities(CapabilitySet::new());
// caps now has tags: "subprotocol:0x0400", "subprotocol:0x0500", etc.

// Other nodes discover migration-capable targets via the index
let filter = SubprotocolRegistry::capability_filter_for(0x0500);
let targets = capability_index.query(&filter);
```

## Migration Handler

`MigrationSubprotocolHandler` dispatches inbound migration messages (0x0500) to the `MigrationOrchestrator`, `MigrationSourceHandler`, or `MigrationTargetHandler` as appropriate. It produces outbound messages with correct destination routing. See [COMPUTE.md](COMPUTE.md) for the full migration protocol.

## Source Files

| File | Purpose |
|------|---------|
| `subprotocol/descriptor.rs` | `SubprotocolDescriptor`, `SubprotocolVersion` |
| `subprotocol/registry.rs` | `SubprotocolRegistry`, ID-to-handler mapping, `enrich_capabilities()` |
| `subprotocol/negotiation.rs` | `SubprotocolManifest`, `NegotiatedSet`, version negotiation |
| `subprotocol/migration_handler.rs` | `MigrationSubprotocolHandler`, migration message dispatch |
