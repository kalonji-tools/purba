// Print the full name of each test that passed in a Jest JSON report.
const fs = require('node:fs')
const report = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'))
for (const file of report.testResults) {
  for (const t of file.assertionResults) {
    if (t.status === 'passed') console.log(`ran: ${t.fullName}`)
  }
}
