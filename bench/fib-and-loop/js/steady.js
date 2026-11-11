// Steady state, not a whole program: each benchmark run many times inside one Node, after
// warm-up, timed with process.hrtime. Prints the answers, then the mean, standard deviation
// and min.
function fib(n) {
  if (n < 2) return n;
  return fib(n - 1) + fib(n - 2);
}
function loop(n) {
  let sum = 0;
  for (let i = 1; i <= n; i++) sum += i;
  return sum;
}
function measure(name, work, input, warm, runs) {
  let answer;
  for (let k = 0; k < warm; k++) answer = work(input);
  const times = [];
  for (let k = 0; k < runs; k++) {
    const start = process.hrtime.bigint();
    answer = work(input);
    times.push(Number(process.hrtime.bigint() - start) / 1e9);
  }
  const mean = times.reduce((a, b) => a + b) / runs;
  const sd = Math.sqrt(times.reduce((a, t) => a + (t - mean) ** 2, 0) / (runs - 1));
  console.log(`${name} ${answer} mean ${mean.toFixed(4)} sd ${sd.toFixed(4)} min ${Math.min(...times).toFixed(4)}`);
}
measure("fib", fib, Number(process.argv[2] ?? 35), 5, 20);
measure("loop", loop, Number(process.argv[3] ?? 100_000_000), 5, 20);
