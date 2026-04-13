# Net

Loosely inspired by the Net from Cyberpunk 2077 - a flat, encrypted mesh where every device is a first-class node. Not affiliated with CD Projekt Red or R. Talsorian Games. This is an engineering take on the concept, not a licensed adaptation.

## What it is

Net is a latency-first encrypted mesh network. Every computer, device, and application is an equal node on a flat topology. There are no clients, no servers, no coordinators. The mesh propagates state, not connections.

## Why not best-effort

ARPANET assumed scarcity. The Net assumes abundance.

ARPANET was designed during the Cold War. Nuclear war was a real possibility. The network had to survive partial destruction, so TCP guarantees delivery: packets will arrive, eventually, in order, even if half the routers are gone. Nodes were scarce. Bandwidth was scarce. Routes were scarce. Every packet was precious because the infrastructure might not be there for the next one. That was the right design for 1969.

It's the wrong design for now. Data is abundant. Nodes are abundant. Bandwidth is abundant. And the external pressure is constant, overwhelming, and unyielding - sensors don't pause, token streams don't wait, market feeds don't care that your queue is full. The firehose doesn't have a pause button. In a world of scarcity, guaranteeing delivery is a virtue. In a world of abundance, guaranteeing delivery is a threat - you're promising to deliver data that will bury the receiver. The bottleneck isn't delivery - it's processing. A delivery guarantee is only as good as the weakest node in the chain. TCP guarantees delivery to a buffer. It doesn't guarantee the receiver can act on it in time. The guarantee creates false confidence: arrival does not equal usefulness.

TCP also presumes that most actors are good. The protocol is cooperative - it assumes both sides want the connection to succeed, that routers will forward honestly, that congestion control will be respected. The entire internet is built on this assumption. It works until it doesn't.

Net makes no presumptions about actors. The only axiom is self-preservation: a node must survive by not getting overloaded. Everything else follows from that. Nodes drop what they can't handle. Relay nodes can't read what they forward. Capability announcements can be verified against behavior. Trust isn't assumed - it's derived from observation. A node that claims capacity it doesn't have will be routed around when its silence or latency betrays it.

Net inverts the default. TCP starts with trust and detects abuse. Net starts with zero assumptions and lets trust emerge from consistent behavior.

Nodes reject work they can't process within a time window. Dropping a packet and re-requesting from a faster node costs nanoseconds across the mesh. Waiting for a congested node's guaranteed response costs milliseconds. When dropping is cheaper than waiting, delivery guarantees become overhead.

This economic inversion has a physical consequence at the queue level.

## Queues

In best-effort networks, queues are a virtue. They absorb bursts, smooth jitter, and let slow consumers catch up. TCP's entire flow control model depends on buffers at every hop. Routers queue. Kernels queue. Applications queue. The queue is what makes the delivery guarantee possible - data waits in line until the receiver is ready.

In latency-first networks, queues are failure. Every nanosecond a packet sits in a queue is latency added. A queue means a node accepted work it couldn't immediately process - it violated the self-preservation axiom. In Net, the queue is a ring buffer with a fixed capacity. When it's full, old data is evicted or new data is dropped. There is no unbounded growth. The queue is a speed buffer, not a waiting room.

This creates an incompatibility. Latency-first systems cannot coexist with best-effort systems on the same transport. A latency-first node operating at 10M+ events/sec will fill a best-effort system's kernel socket buffer before a single context switch can happen. The best-effort node hasn't even woken up to read the queue and the queue is already full. From the best-effort node's perspective, this looks like a flood attack. From the latency-first node's perspective, it sent normal traffic to a node that couldn't keep up.

The mismatch is fundamental. Best-effort systems use queues to decouple producers from consumers in time. Latency-first systems require producers and consumers to operate at the same timescale. When they meet, the latency-first system overwhelms the best-effort system's buffers, and the best-effort system's backpressure signals (TCP window scaling, congestion control) operate orders of magnitude too slowly to matter. By the time TCP tells the sender to slow down, the sender has already moved on.

This is why Net runs its own transport (BLTP over UDP) rather than layering on TCP. It's not an optimization. It's a necessity. The two models are physically incompatible at the queue level.

## Properties

**Latency-first.** The entire stack is designed so that "within the time window" is measured in nanoseconds. Sub-nanosecond header serialization. Nanosecond-scale heartbeats, forwarding hops, and failure recovery. The floor is low enough that packet scheduling operates at timescales traditionally reserved for local function calls. See [Benchmarks](#benchmarks) for measured numbers.

**Streaming-first.** Data is a continuous flow, not documents. The event bus, sharded ring buffers, and adaptive batching assume data is produced incrementally and consumed incrementally. There are no requests and responses. There are streams.

**Zero-copy.** Ring buffers, no garbage collector, native Rust. Forwarding doesn't allocate or copy payload data. This is what makes the per-hop numbers possible and it's a design principle, not just an optimization.

**Encrypted end-to-end.** Noise protocol handshakes for key exchange. ChaCha20-Poly1305 authenticated encryption. Every packet is encrypted between source and destination. Intermediate nodes never see plaintext.

**Untrusted relay.** Nodes forward packets without decrypting payloads. The mesh can route through infrastructure you don't trust. Combined with E2E encryption, this means the network can grow through adversarial nodes.

**Schema-agnostic.** The transport layer moves bytes, not structures. A raw event is payload plus hash. The protocol never inspects content. Structure is optional and emerges where participants agree on it - two nodes can negotiate typed, ordered streams while the rest of the mesh passes opaque bytes.

**Optionally ordered.** Ordering is per-stream, not global. The unordered path is the fast path. Causal ordering is available when streams need it. The cost of ordering is paid only by streams that require it.

**Optionally typed.** The protocol doesn't care what's in the payload. The behavior plane can. Typing is a local agreement between nodes, not a network requirement.

**Native backpressure.** Nodes drop packets without reply. This isn't a failure mode - it's the design. The proximity graph makes silence a signal, not an error. A node that stops responding doesn't need to send an error. Its neighbors already know within a heartbeat interval.

## Backpressure

In TCP, backpressure is negotiated. The receiver advertises a window size. The sender respects it. If the receiver is slow, the window shrinks. If the network is congested, both sides back off. This takes round trips. At internet scale, that's milliseconds of negotiation before a single byte is throttled.

In Net, backpressure is immediate and unilateral. A node that can't keep up stops processing. That's it. There's no window advertisement, no negotiation, no round trip. The node's ring buffer is full, so new data either evicts the oldest entry or gets dropped at the boundary. The node doesn't tell anyone it's overwhelmed. It just goes silent on that stream.

Silence propagates through the proximity graph. Neighbors observe the silence within a heartbeat interval. The failure detector marks the node as degraded. The circuit breaker trips. From this point, no new traffic is routed to that node.

The critical difference: TCP backpressure slows the sender down. Net backpressure routes around the problem. The sender doesn't slow down. It doesn't need to. The mesh has other nodes.

## Rerouting

When a node goes silent - whether from overload, failure, or a deliberate kill-switch - the mesh reroutes in three phases:

**Detection.** The failure detector runs on heartbeats at nanosecond-scale granularity. A missed heartbeat triggers a status check. The node is marked suspect. After the timeout threshold, the circuit breaker opens. Total time from last heartbeat to confirmed failure: nanoseconds to low microseconds, depending on configuration.

**Recovery.** The recovery manager evaluates alternates in nanoseconds. It looks at the routing table, finds nodes with matching capabilities, and selects the best candidate based on proximity, load, and latency estimates. A full fail-and-recover cycle completes in sub-microsecond time.

**Cutover.** The routing table updates atomically. The next packet on that stream goes to the new node. There is no drain period, no graceful handoff, no connection migration. The state was never tied to the old node's connection - it was in the event stream. The new node picks up from where the stream left off.

From the sender's perspective, nothing happened. One packet went to node A, the next went to node B. The sender didn't decide this. The mesh did. The sender doesn't know or care which node is handling its traffic. It sent data into the mesh and the mesh delivered it to a node that could process it in time.

From the receiver's perspective, the stream continues. If ordering is enabled, the causal chain is intact - every event links to its parent hash. If ordering is disabled, events arrive from wherever the mesh routes them. Either way, the stream didn't stop. A node died and the stream didn't notice.

This is what "propagates state, not connections" means in practice. The connection to node A is gone. The state is on node B. Nothing was lost, nothing was retried, and the reroute was scheduled faster than a context switch on the dead node's operating system.

## Topology

**Flat mesh.** All nodes are protocol-equal. No node has special authority. But nodes aren't equal in capability - the capability system makes that explicit. Every node is a first-class citizen, but not every node can serve every request. The mesh routes based on what nodes can do, not what they are.

**Swarm discovery.** No DNS, no bootstrap servers, no service registry. Nodes discover each other directly through the mesh itself, the same way planets find each other - by existing in proximity and being observable.

**Pingwave.** Nodes emit periodic heartbeats that propagate outward within a hop radius. If you can hear a node's pingwave, you know it exists, how far away it is, and what it can do.

**Proximity graph.** Each node maintains a local view of its neighborhood, not a global directory. The graph is built from direct observation and derivation from neighbors' observations. A node doesn't need to see every other node. It observes enough to derive the rest.

**Capability announcements.** Nodes advertise what they can do - hardware, models, tools, capacity. The mesh uses this for routing decisions. Compute-heavy workloads fall toward GPU-rich nodes the way mass curves spacetime toward other mass.

## Consistency

**Observational.** There is no global truth, only local views. Each node observes its neighborhood and derives the state of the wider mesh from those observations. Two nodes may disagree about the mesh state at a given instant. That's fine. Their views are causally consistent within their own observation window.

No global consensus. No coordinator. No privileged node. Consistency emerges from causal ordering and the speed of propagation.

Everything in the mesh is either observable or derivable. A node doesn't need direct heartbeats from every peer. If it knows a gateway is alive and the gateway reports on its subnet, the subnet's state is derived. The mesh is an inference engine about its own state, not just a forwarding engine for events.

The only authority the mesh respects is physics. Propagation speed, causal ordering, the speed of light. Everything else is negotiable.

## State, not connections

Traditional networking treats the connection as the primary object. You establish a socket, maintain it, tear it down. If the connection breaks, the relationship breaks.

Net propagates state. Connections are ephemeral transport - the current shortest path between where state is and where it needs to be. When a path breaks, state doesn't wait for recovery. It moves. The routing table updates, the proximity graph adjusts, the state continues on a different path. No reconnection, no session resumption, no handshake retry. Identity lives in the state chain, not in the socket.

## Invariants

**Identity.** A node is its keypair. Every node has a long-lived cryptographic identity - the public key is the node ID, the private key is the authority to act as that node. Identity is cryptographic, not topological. A node can roam across networks, change IPs, traverse NAT, switch interfaces, and remain the same node. All communication is authenticated against this identity, independent of network location.

**Bootstrap.** Nodes require at least one known peer to join the mesh. This initial contact is out-of-band - a manual address, LAN broadcast, QR code, config file, cached peers from a previous session. After first contact, discovery is emergent. Pingwaves propagate, the proximity graph builds, and the mesh takes over. "No bootstrap servers" doesn't mean no first contact. It means the protocol doesn't depend on any fixed infrastructure to function after that first handshake.

**Scale.** The mesh is logically flat but scales via hierarchical summarization. At small scale, nodes observe each other directly. At larger scale, nodes form clusters. Gateway nodes aggregate health, compress capability summaries, and propagate state for their subnet. A distant node doesn't need individual heartbeats from every node in a cluster - it observes the gateway and derives the rest. This keeps observation cost bounded: each node observes its peers at its level of the hierarchy, and derivation gives it awareness of everything below.

**Time.** There is no global clock. The system does not require synchronized clocks and has no dependency on NTP or wall-clock agreement. Event ordering is derived from causal relationships - vector clocks, Lamport timestamps, parent hashes in the event chain. Two nodes may assign different wall-clock times to the same event. That doesn't matter. What matters is that they agree on causal order: what happened before what.

**Participation.** Relay is a cost of participation. If you're on the mesh, you forward traffic within your resource limits. This is cooperative, not economic - the current design assumes a mesh of your own machines and trusted participants. Incentive mechanisms for public, multi-party, or adversarial meshes are out of scope. Nodes enforce their own relay limits through the same autonomy rules that govern everything else.

## Device autonomy

Every node sets its own rules. A node can rate-limit, reject, redirect, or kill-switch independently. The mesh doesn't override a node's sovereignty. Autonomy rules, safety envelopes, and resource limits are local decisions, not network policy.

## A new class of systems

Existing networking falls into two categories:

**Best-effort networks** (TCP/IP, HTTP, gRPC). Optimized for delivery. Queues absorb bursts. Backpressure is negotiated. Connections are stateful. Consistency is global or eventual. Trust is assumed. The sender slows down when the receiver can't keep up. This model dominates because it was designed first and everything was built on top of it.

**Real-time networks** (CAN bus, EtherCAT, TSN, military MANETs). Optimized for deterministic timing. Fixed topologies. Dedicated hardware. Time-slotted access. These achieve low latency by controlling the physical layer - you get guaranteed timing because you own the wire. They don't scale to heterogeneous, dynamic, adversarial environments because they can't. The guarantees depend on controlling the infrastructure.

Net is neither. It achieves real-time latencies on commodity hardware over commodity networks without controlling the physical layer. It doesn't guarantee timing through infrastructure control - it guarantees it through architectural choices: drop instead of queue, route around instead of wait, observe instead of coordinate, derive instead of query.

The pieces exist independently as solved problems. Event sourcing (Kafka). Process migration (Erlang/OTP). Distributed state (CRDTs). Capability scheduling (Kubernetes). Self-healing mesh (military MANETs). Causal ordering (vector clocks). Nobody composed them into a single runtime at nanosecond speeds because nobody had a transport layer fast enough. You can't migrate state in microseconds if your network adds milliseconds. You can't detect failure in nanoseconds if your heartbeat protocol runs over TCP.

The benchmark numbers aren't performance metrics. They're existence proofs. They measure packet scheduling - the time to process, route, and queue a packet for transmission, not the wire time. But they demonstrate that the software layer is no longer the bottleneck. The scheduling overhead per packet is nanoseconds. The remaining latency is physics: NIC, wire, speed of light. The software got out of the way.

This is the gap: a system that operates at hardware timescales, on commodity hardware, across untrusted infrastructure, with no central coordination, no global consensus, and no assumptions about the goodwill of participants. Best-effort networks can't do this because their queue model is incompatible. Real-time networks can't do this because their guarantees require owning the wire. Net sits in the space between them - fast enough to be real-time, open enough to be general-purpose, hostile enough to survive the actual internet.

## Processing without storage

In every existing system, the node that processes data is the node that stores it, or the node that stores it is a known, fixed location. Databases, message brokers, file systems - processing and storage are co-located or connected by a stable, long-lived path. If the storage node dies, you fail over to a replica. If the processing node dies, you restart it and reconnect to storage.

Net doesn't have this coupling. The event bus is a ring buffer - it's a speed buffer, not storage. The adapters (Redis, JetStream) are optional persistence layers, not requirements. A node processes events in flight. It doesn't store them unless explicitly configured to. The event stream flows through nodes like current through a wire - the wire doesn't remember the electricity.

This means processing can happen anywhere without first solving "where is the data." The data is in the stream. The stream is on the mesh. Any node with matching capabilities can pick up the stream and process it. If that node dies, another node picks it up. Neither node "had" the data. The data was passing through.

Storage becomes a choice, not an assumption. A node can choose to persist events to Redis. A node can choose to replay from JetStream. But the mesh itself doesn't require storage to function. Events exist in the ring buffers of the nodes they're passing through, for as long as they're relevant, and then they're gone. If you need them later, that's what the persistence adapters are for. But the processing path - the hot path - never touches disk, never queries a database, never waits on storage I/O.

This is why the latency numbers are what they are. Processing isn't waiting on storage. Storage isn't blocking processing. They're independent decisions made by independent nodes.

## Cost of devices

When processing can be offloaded to the mesh, edge devices don't need to be smart. They need to be present.

A sensor node doesn't need a GPU to run inference. It needs a network interface and a microcontroller. It streams raw data into the mesh and the mesh routes the processing to a node that has the capability. A camera doesn't need to run object detection. A thermostat doesn't need to run a language model. A brake sensor doesn't need to run path planning. They produce data. The mesh finds compute.

The entire transport library - Noise protocol, ChaCha20-Poly1305 encryption, routing, swarm discovery, failure detection, capability system - compiles to 914KB stripped. Under a megabyte. It fits on anything with a network interface.

This inverts the economics of edge deployment. Today, every device that needs intelligence must contain intelligence - or pay for a round trip to a cloud that does. That means expensive hardware at the edge, or latency to a data center, or both. Net eliminates this choice. Devices can be cheap, dumb, and deterministic. They do one thing well - sense, actuate, relay - and the mesh provides the intelligence dynamically.

The capability announcement system is what makes this work. A $5 sensor node advertises that it produces temperature data. A $500 GPU node three hops away advertises that it runs inference models. The mesh connects them automatically. The sensor node didn't need to know the GPU node exists. The GPU node didn't need to be configured to accept sensor data. The capability graph brought them together.

This means you can scale a deployment by adding cheap nodes for coverage and a few expensive nodes for compute. The ratio adjusts dynamically - add more sensors and the compute nodes absorb the load. Add more compute and the sensors' data gets processed faster. Neither side needs to be reconfigured. The mesh adapts.

## Applications

**AI runtime.** The original use case. Token streams, tool-call results, guardrail decisions, and consensus votes flowing across heterogeneous GPU nodes. Compute-heavy inference routes to whichever node has capacity. The mesh is the runtime - no orchestrator dispatching work, no queue broker mediating between models.

**Vehicular sensor mesh.** Cars sharing LIDAR, radar, and camera feeds across a local swarm. A vehicle that can't see around a corner derives it from a neighbor that can. Processing - object detection, path planning - routes to whichever vehicle or roadside unit has spare capacity. Vehicles also sync intent - braking, turning, route changes - so every car in the swarm knows what the vehicle ahead will do before it does it. Brake lights are a 200ms visual signal processed by a human. An intent stream on the mesh is scheduled in nanoseconds and processed by software. The car behind doesn't react to braking. It knows about the braking before the brake pads touch the rotor.

**Robotics factory floor.** Robots don't need line-of-sight for networking. The mesh routes through whatever nodes are reachable. A robot behind a steel column relays through one that isn't. If a robot goes offline, the mesh schedules a reroute in sub-microsecond time - the assembly line doesn't stop. No WiFi access points, no central controller, no single point of failure.

**Edge compute.** IoT devices, phones, single-board computers acting as equal peers. A sensor node that can't run inference locally routes to the nearest node that can. Capability announcements make this automatic - the mesh knows what every node can do and routes accordingly.

**Local-first collaboration.** Devices on the same LAN forming a mesh without cloud infrastructure. Pingwave bootstrap on the local network, no configuration, no accounts, no servers. The mesh exists for as long as the devices are in proximity.

**Disaster response.** Phones, drones, portable radios forming a mesh with no surviving infrastructure. Each device contributes what it has. A phone relays. A drone provides compute. A satellite uplink node becomes a gateway. The mesh forms from whatever is present and routes around whatever is gone.

**Remote surgery.** A surgeon operating remotely, with control signals and haptic feedback routed across the mesh. The robot doesn't need a dedicated fiber link to one server. If the primary compute node lags, the mesh reroutes to another mid-operation. The surgeon doesn't notice. The patient doesn't notice. The scalpel doesn't stop.

**Drone swarms.** Coordinated flight without a ground controller. Each drone shares position, velocity, and intent. Formation changes propagate across the swarm faster than aerodynamic forces alter the flight path. A drone that loses a motor broadcasts the failure; the swarm adjusts formation before the drone has begun to fall.

**Live performance.** Lighting, audio, video, and pyrotechnics synchronized across hundreds of nodes on a stage rig. A DMX controller dies, another node picks up the cue list. No show stop. Latency low enough that audio sync across the mesh is tighter than the speed of sound across the venue.

**Precision agriculture.** Tractors, drones, soil sensors, and weather stations forming a field mesh. A tractor that detects a soil condition shares it, and every other tractor adjusts its seeding or irrigation without routing through a cloud service. The field is the network.

**Multiplayer gaming.** No dedicated server. Players' devices form the mesh. Game state propagates peer-to-peer with causal ordering. A player drops, the mesh reroutes. Capability-aware routing means heavier computation - physics, collision, world state - routes toward the gaming PC, not the phone. The weakest device doesn't become the bottleneck; the mesh routes around its limitations the same way it routes around any other constraint. Ping is meaningless here - there's no fixed server to round-trip to. The relevant measurement is observation latency: the time from when a state change is produced to when another node can observe it. A one-way traversal through however many hops the mesh chose, which can change packet to packet. That's the light travel time of this network.

## Benchmarks

All numbers below measure **packet scheduling** - the time to process, route, encrypt, and queue a packet for transmission. They do not include NIC transfer, wire latency, or speed-of-light propagation. The software layer is what these benchmarks prove is no longer the bottleneck.

**Test system:** Apple M1 Max, macOS.

### Routing

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Header serialize | 0.57 ns | **1.74G ops/sec** |
| Header deserialize | 0.94 ns | **1.07G ops/sec** |
| Routing lookup (hit) | 14.92 ns | **67.0M ops/sec** |
| Routing lookup (miss) | 11.96 ns | **83.6M ops/sec** |
| Decision pipeline (parse + lookup + forward) | 15.34 ns | **65.2M ops/sec** |

### Multi-hop Forwarding

| Hops | Latency | Throughput |
|-----:|---------|------------|
| 1 | 57.99 ns | **17.2M ops/sec** |
| 2 | 113.61 ns | **8.80M ops/sec** |
| 3 | 159.94 ns | **6.25M ops/sec** |
| 5 | 276.92 ns | **3.61M ops/sec** |

| Threads | Forwarding Throughput |
|--------:|----------------------:|
| 4 | 4.67 M/s |
| 8 | 7.48 M/s |
| 16 | 9.70 M/s |

### Failure Detection & Recovery

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Heartbeat (existing node) | 29.22 ns | **34.2M ops/sec** |
| Status check | 12.22 ns | **81.9M ops/sec** |
| Circuit breaker check | 13.73 ns | **72.8M ops/sec** |
| Recovery (evaluate alternates) | 278.80 ns | **3.59M ops/sec** |
| Full fail + recover cycle | 312.53 ns | **3.20M ops/sec** |

### Swarm / Discovery

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Pingwave serialize | 0.80 ns | **1.25G ops/sec** |
| Pingwave roundtrip | 0.94 ns | **1.06G ops/sec** |
| New peer discovery | 126.60 ns | **7.90M ops/sec** |

| Nodes | Graph query | Path within hops |
|------:|------------:|-----------------:|
| 100 | 2.30 us | 2.76 us |
| 500 | 7.35 us | 9.30 us |
| 1,000 | 120.31 us | 122.69 us |
| 5,000 | 173.43 us | 192.91 us |

### Encryption

| Payload | ChaCha20-Poly1305 | XChaCha20-Poly1305 |
|--------:|------------------:|-------------------:|
| 64B | 709 ns | 1,207 ns |
| 256B | 1,284 ns | 1,790 ns |
| 1KB | 3,486 ns | 4,002 ns |
| 4KB | 12,440 ns | 12,980 ns |

### Capability System

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Filter (single tag) | 10.02 ns | **99.8M ops/sec** |
| Filter (require GPU) | 4.07 ns | **245.5M ops/sec** |
| GPU check | 0.32 ns | **3.13G ops/sec** |
| Capability announcement create | 388.80 ns | **2.57M ops/sec** |

| Nodes | Tag query | Complex query |
|------:|----------:|--------------:|
| 1,000 | 12.69 us | 41.08 us |
| 5,000 | 72.20 us | 211.51 us |
| 10,000 | 170.05 us | 512.64 us |
| 50,000 | 2.58 ms | 4.57 ms |

### Multi-threaded Scaling

| Threads | Legacy Pool | Fast Pool | Speedup |
|--------:|------------:|----------:|--------:|
| 8 | 2.33 M/s | **8.68 M/s** | **3.7x** |
| 16 | 1.82 M/s | **9.28 M/s** | **5.1x** |
| 32 | 1.78 M/s | **9.90 M/s** | **5.6x** |

Pool contention (acquire/release):

| Threads | Legacy Pool | Fast Pool | Speedup |
|--------:|------------:|----------:|--------:|
| 8 | 4.34 M/s | **117.1 M/s** | **27x** |
| 16 | 4.34 M/s | **125.1 M/s** | **29x** |
| 32 | 4.17 M/s | **135.5 M/s** | **32x** |

### SDK Ingestion

| SDK | Method | Throughput | Latency |
|-----|--------|------------|---------|
| Go | IngestRaw (9B) | **2.65M/sec** | 377 ns |
| Go | Batch (1000) | **2.44M/sec** | 411 ns/event |
| Python | ingest_raw (9B) | **2.53M/sec** | 0.40 us |
| Python | Batch (1000) | **2.78M/sec** | 0.36 us |
| Node.js | pushBatch | **2.89M/sec** | 0.35 us |
| Node.js | push (single) | **2.26M/sec** | 0.44 us |
| Bun | Batch (1000) | **3.37M/sec** | 0.30 us |
| Bun | push (single) | **2.05M/sec** | 0.49 us |

All SDKs exceed **2M events/sec** with optimal ingestion patterns. Go achieves zero allocations on raw ingestion. Node.js sync methods are 31x faster than async. Bun batch ingestion is ~17% faster than Node.js.
