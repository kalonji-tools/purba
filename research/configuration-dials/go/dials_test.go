// Tests that research/configuration-dials/go/run.sh runs under each dial.
package dials

import (
	"fmt"
	"testing"
	"time"
)

func TestSleepsTwoSeconds(t *testing.T) {
	time.Sleep(2 * time.Second)
}

func TestLongUnlessShort(t *testing.T) {
	if testing.Short() {
		t.Skip("probe: skipped under -short")
	}
	t.Fatal("probe: the long path fails")
}

func TestVetFinding(t *testing.T) {
	_ = fmt.Sprintf("%d", "a string where vet expects an integer")
}
