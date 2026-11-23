// Tests that research/configuration-dials/jest/run.sh runs under each dial.
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
let attempts = 0;

test("waits one second", async () => {
  await sleep(1000);
  expect(true).toBe(true);
});

test("waits one second, marked five", async () => {
  await sleep(1000);
  expect(true).toBe(true);
}, 5000);

test("fails on its first attempt", () => {
  attempts += 1;
  expect(attempts).toBeGreaterThan(1);
});

test("matches a snapshot", () => {
  expect({ answer: 42 }).toMatchSnapshot();
});
