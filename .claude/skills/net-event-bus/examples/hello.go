// Minimal sanity check for the Go binding.
// Run: go run hello.go
//
// What it proves: the binding loads, a node starts, you can ingest JSON,
// poll it back, and shutdown is clean. Go has no async iterator and no
// named-channel API — you write the poll loop yourself.

package main

import (
	"fmt"
	"log"
	"time"

	"github.com/ai-2070/net/net/crates/net/bindings/go/net"
)

func main() {
	bus, err := net.New(&net.Config{NumShards: 1})
	if err != nil {
		log.Fatal(err)
	}
	defer bus.Shutdown()

	if err := bus.IngestRaw(`{"msg":"hello, mesh"}`); err != nil {
		log.Fatal(err)
	}

	// Tiny wait for the drain worker to make the event visible to Poll.
	time.Sleep(20 * time.Millisecond)

	resp, err := bus.Poll(10, "")
	if err != nil {
		log.Fatal(err)
	}
	if len(resp.Events) == 0 {
		log.Fatal("no events received")
	}
	// resp.Events is []json.RawMessage (=[][]byte). Print as string,
	// otherwise fmt.Println renders the raw bytes as `[123 34 ...]`.
	fmt.Println("received:", string(resp.Events[0]))
}
