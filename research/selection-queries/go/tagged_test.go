//go:build slow && !nodb

package sel

import "testing"

// A build constraint selects this file. Its line is a boolean expression.
func TestTaggedSlow(t *testing.T) {}
