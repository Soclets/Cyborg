// Purpose: a loop with an add, with the count from the command line.
const n = Number(process.argv[2] ?? 100_000_000);
let sum = 0;
for (let i = 1; i <= n; i++) sum += i;
console.log(sum);
