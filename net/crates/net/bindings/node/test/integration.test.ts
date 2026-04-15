/**
 * Integration tests for Net Node.js bindings with NLTP.
 *
 * Run tests:
 *   npm test
 *
 * Environment variables:
 *   RUN_INTEGRATION_TESTS - Set to "1" to run integration tests
 */

import { describe, it, expect, beforeAll } from "vitest";
import {
  Net,
  StoredEvent,
  PollResponse,
  generateNltpKeypair,
  NltpKeypair,
} from "../index";
import * as crypto from "crypto";

const RUN_INTEGRATION_TESTS = process.env.RUN_INTEGRATION_TESTS === "1";

function uniquePrefix(): string {
  return `test_${Date.now()}_${Math.random().toString(36).slice(2)}`;
}

async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Check if NLTP feature is available
let nltpAvailable = false;
try {
  if (typeof generateNltpKeypair === "function") {
    generateNltpKeypair();
    nltpAvailable = true;
  }
} catch {
  nltpAvailable = false;
}

describe.skipIf(!RUN_INTEGRATION_TESTS || !nltpAvailable)(
  "NLTP Integration Tests",
  () => {
    it("should generate valid keypairs", () => {
      const keypair = generateNltpKeypair();

      expect(keypair).toBeDefined();
      expect(keypair.publicKey).toBeDefined();
      expect(keypair.secretKey).toBeDefined();

      // Keys should be 32 bytes hex-encoded (64 hex chars)
      expect(keypair.publicKey.length).toBe(64);
      expect(keypair.secretKey.length).toBe(64);

      // Should be valid hex
      expect(/^[0-9a-f]+$/i.test(keypair.publicKey)).toBe(true);
      expect(/^[0-9a-f]+$/i.test(keypair.secretKey)).toBe(true);

      // Each call should generate different keypairs
      const keypair2 = generateNltpKeypair();
      expect(keypair2.publicKey).not.toBe(keypair.publicKey);
      expect(keypair2.secretKey).not.toBe(keypair.secretKey);
    });

    it("should exchange events between initiator and responder", async () => {
      // Generate keypair for responder
      const responderKeypair = generateNltpKeypair();

      // Generate shared PSK (32 bytes hex)
      const psk = crypto.randomBytes(32).toString("hex");

      // Create responder (binds first, waits for initiator)
      const responder = await Net.create({
        numShards: 1,
        nltp: {
          bindAddr: "127.0.0.1:19000",
          peerAddr: "127.0.0.1:19001",
          psk: psk,
          role: "responder",
          secretKey: responderKeypair.secretKey,
          publicKey: responderKeypair.publicKey,
          reliability: "light",
        },
      });

      // Small delay to ensure responder is ready
      await sleep(50);

      // Create initiator
      const initiator = await Net.create({
        numShards: 1,
        nltp: {
          bindAddr: "127.0.0.1:19001",
          peerAddr: "127.0.0.1:19000",
          psk: psk,
          role: "initiator",
          peerPublicKey: responderKeypair.publicKey,
          reliability: "light",
        },
      });

      try {
        // Wait for handshake to complete
        await sleep(200);

        // Initiator sends events to responder
        for (let i = 0; i < 5; i++) {
          await initiator.ingestRaw(
            JSON.stringify({ source: "initiator", index: i }),
          );
        }

        // Responder sends events to initiator
        for (let i = 0; i < 5; i++) {
          await responder.ingestRaw(
            JSON.stringify({ source: "responder", index: i }),
          );
        }

        // Wait for events to propagate
        await sleep(500);

        // Poll from both sides
        const initiatorEvents = await initiator.poll({ limit: 100 });
        const responderEvents = await responder.poll({ limit: 100 });

        // Both should have received events
        expect(initiatorEvents.events.length).toBeGreaterThan(0);
        expect(responderEvents.events.length).toBeGreaterThan(0);
      } finally {
        await initiator.shutdown();
        await responder.shutdown();
      }
    });

    it("should support batch ingestion over NLTP", async () => {
      const responderKeypair = generateNltpKeypair();
      const psk = crypto.randomBytes(32).toString("hex");

      const responder = await Net.create({
        numShards: 1,
        nltp: {
          bindAddr: "127.0.0.1:19002",
          peerAddr: "127.0.0.1:19003",
          psk: psk,
          role: "responder",
          secretKey: responderKeypair.secretKey,
          publicKey: responderKeypair.publicKey,
        },
      });

      await sleep(50);

      const initiator = await Net.create({
        numShards: 1,
        nltp: {
          bindAddr: "127.0.0.1:19003",
          peerAddr: "127.0.0.1:19002",
          psk: psk,
          role: "initiator",
          peerPublicKey: responderKeypair.publicKey,
        },
      });

      try {
        await sleep(200);

        // Batch ingest
        const events: string[] = [];
        for (let i = 0; i < 20; i++) {
          events.push(JSON.stringify({ batchIndex: i }));
        }
        const count = await initiator.ingestRawBatch(events);
        expect(count).toBe(20);

        await sleep(500);

        const response = await responder.poll({ limit: 100 });
        expect(response.events.length).toBeGreaterThan(0);
      } finally {
        await initiator.shutdown();
        await responder.shutdown();
      }
    });

    it("should work with full reliability mode", async () => {
      const responderKeypair = generateNltpKeypair();
      const psk = crypto.randomBytes(32).toString("hex");

      const responder = await Net.create({
        numShards: 1,
        nltp: {
          bindAddr: "127.0.0.1:19004",
          peerAddr: "127.0.0.1:19005",
          psk: psk,
          role: "responder",
          secretKey: responderKeypair.secretKey,
          publicKey: responderKeypair.publicKey,
          reliability: "full",
          heartbeatIntervalMs: 1000,
          sessionTimeoutMs: 10000,
        },
      });

      await sleep(50);

      const initiator = await Net.create({
        numShards: 1,
        nltp: {
          bindAddr: "127.0.0.1:19005",
          peerAddr: "127.0.0.1:19004",
          psk: psk,
          role: "initiator",
          peerPublicKey: responderKeypair.publicKey,
          reliability: "full",
          heartbeatIntervalMs: 1000,
          sessionTimeoutMs: 10000,
        },
      });

      try {
        await sleep(200);

        // Send events with full reliability
        for (let i = 0; i < 10; i++) {
          await initiator.ingestRaw(JSON.stringify({ reliable: true, seq: i }));
        }

        await sleep(500);

        const response = await responder.poll({ limit: 100 });
        expect(response.events.length).toBeGreaterThan(0);
      } finally {
        await initiator.shutdown();
        await responder.shutdown();
      }
    });
  },
);
