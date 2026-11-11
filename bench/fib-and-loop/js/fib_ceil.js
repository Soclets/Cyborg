// Ceil: as fast as Node can go. fib(35) by fast doubling.
function pair(n) { // [fib(n), fib(n + 1)]
  if (n === 0) return [0, 1];
  const [a, b] = pair(Math.floor(n / 2));
  const c = a * (2 * b - a), d = a * a + b * b;
  return n % 2 ? [d, c + d] : [c, d];
}
console.log(pair(35)[0]);
