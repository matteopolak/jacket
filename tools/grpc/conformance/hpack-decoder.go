// Usage: hpack-decoder RECORDS_JSONL (the owning HPACK fixture output).
// Independently decode every emitted block, preserving each story's HPACK context.
// Build inside the pinned grpc-go oracle module to use its exact x/net dependency.
package main

import (
	"bufio"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"

	"golang.org/x/net/http2/hpack"
)

type record struct {
	Story      string `json:"story"`
	Index      int    `json:"index"`
	Decoded    bool   `json:"decoded"`
	Encoded    string `json:"encoded"`
	TableLimit uint32 `json:"table_limit"`
}
type story struct {
	Cases []struct {
		Headers []map[string]string `json:"headers"`
	} `json:"cases"`
}

func main() {
	if len(os.Args) != 2 {
		panic("supply emitted JSON-lines path")
	}
	file, err := os.Open(os.Args[1])
	if err != nil {
		panic(err)
	}
	defer file.Close()
	scanner := bufio.NewScanner(file)
	scanner.Buffer(make([]byte, 4096), 1<<26)
	current := ""
	var original story
	var decoder *hpack.Decoder
	cases, failures, stories := 0, 0, 0
	for scanner.Scan() {
		var row record
		if err := json.Unmarshal(scanner.Bytes(), &row); err != nil {
			panic(err)
		}
		if row.Story != current {
			current = row.Story
			stories++
			bytes, err := os.ReadFile(current)
			if err != nil {
				panic(err)
			}
			original = story{}
			if err := json.Unmarshal(bytes, &original); err != nil {
				panic(err)
			}
			decoder = hpack.NewDecoder(4096, nil)
			decoder.SetMaxStringLength(1 << 26)
		}
		if row.Index < 0 || row.Index >= len(original.Cases) {
			panic("case outside story")
		}
		decoder.SetAllowedMaxDynamicTableSize(row.TableLimit)
		bytes, err := hex.DecodeString(row.Encoded)
		if err != nil {
			panic(err)
		}
		fields, err := decoder.DecodeFull(bytes)
		expected := original.Cases[row.Index].Headers
		valid := err == nil && len(fields) == len(expected)
		if valid {
			for i, field := range fields {
				if len(expected[i]) != 1 || expected[i][field.Name] != field.Value {
					valid = false
					break
				}
				if _, exists := expected[i][field.Name]; !exists {
					valid = false
					break
				}
			}
		}
		cases++
		if !valid || !row.Decoded {
			failures++
			// Only a corpus-relative identity should appear in a published summary.
			fmt.Fprintf(os.Stderr, "story=%d case=%d own_decode=%t independent_decode=%t error=%v\n", stories-1, row.Index, row.Decoded, valid, err)
		}
	}
	if err := scanner.Err(); err != nil {
		panic(err)
	}
	fmt.Printf("{\"stories\":%d,\"cases\":%d,\"failures\":%d}\n", stories, cases, failures)
	if failures != 0 {
		os.Exit(1)
	}
}
