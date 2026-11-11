// Steady state, not a whole program: each benchmark run many times inside one JVM, after
// warm-up, timed with System.nanoTime, so the JIT has compiled it. Prints the answers, then the
// mean, standard deviation and min.
public class Steady {
    static long fib(long n) {
        if (n < 2) return n;
        return fib(n - 1) + fib(n - 2);
    }

    static long loop(long n) {
        long sum = 0;
        for (long i = 1; i <= n; i++) sum += i;
        return sum;
    }

    interface Work { long run(long input); }

    static void measure(String name, Work work, long input, int warm, int runs) {
        long answer = 0;
        for (int k = 0; k < warm; k++) answer = work.run(input);
        double[] times = new double[runs];
        for (int k = 0; k < runs; k++) {
            long start = System.nanoTime();
            answer = work.run(input);
            times[k] = (System.nanoTime() - start) / 1e9;
        }
        double mean = 0, least = Double.MAX_VALUE;
        for (double t : times) { mean += t; least = Math.min(least, t); }
        mean /= runs;
        double spread = 0;
        for (double t : times) spread += (t - mean) * (t - mean);
        System.out.printf("%s %d mean %.4f sd %.4f min %.4f%n", name, answer, mean,
            Math.sqrt(spread / (runs - 1)), least);
    }

    public static void main(String[] args) {
        long fibInput = args.length > 0 ? Long.parseLong(args[0]) : 35;
        long loopInput = args.length > 1 ? Long.parseLong(args[1]) : 100_000_000L;
        measure("fib", Steady::fib, fibInput, 10, 20);
        measure("loop", Steady::loop, loopInput, 10, 20);
    }
}
