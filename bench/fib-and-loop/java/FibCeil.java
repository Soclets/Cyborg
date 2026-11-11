// Ceil: as fast as Java can go. fib(35) by fast doubling, run with startup-friendly flags.
public class FibCeil {
    static long[] pair(long n) { // {fib(n), fib(n + 1)}
        if (n == 0) return new long[] {0, 1};
        long[] half = pair(n / 2);
        long a = half[0], b = half[1];
        long c = a * (2 * b - a), d = a * a + b * b;
        return n % 2 == 1 ? new long[] {d, c + d} : new long[] {c, d};
    }

    public static void main(String[] args) {
        System.out.println(pair(35)[0]);
    }
}
