package sel

import "testing"

// Go has no tag. "Slow" and "DB" are parts of the test names.
func TestAdd(t *testing.T)           {}
func TestAddSlow(t *testing.T)       {}
func TestQueryDB(t *testing.T)       {}
func TestMigrateDBSlow(t *testing.T) {}

func TestParse(t *testing.T) {
	for _, c := range []string{"a", "b"} {
		t.Run(c, func(t *testing.T) {})
	}
}
