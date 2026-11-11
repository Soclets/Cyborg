// Purpose: the cost of a function call, with n from the command line.
function fib(n) {
  if (n < 2) return n;
  return fib(n - 1) + fib(n - 2);
}
console.log(fib(Number(process.argv[2] ?? 35)));
