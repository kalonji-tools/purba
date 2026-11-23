// Tests that research/configuration-dials/vitest/run.sh runs under each dial.
import { expect, test } from "vitest";

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
let attempts = 0;

test("waits one second", async () => {
  await sleep(1000);
  expect(true, "the test only waits").toBe(true);
});

test("waits one second, marked five", { timeout: 5000 }, async () => {
  await sleep(1000);
  expect(true, "the test only waits").toBe(true);
});

test("fails on its first attempt", () => {
  attempts += 1;
  expect(attempts, "the first attempt fails, a retry passes").toBeGreaterThan(1);
});
