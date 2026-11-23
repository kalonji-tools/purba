// A test marked .only, which research/configuration-dials/vitest/run.sh runs with and without CI.
import { expect, test } from "vitest";

test.only("marked only", () => {
  expect(true, "the test only passes").toBe(true);
});
