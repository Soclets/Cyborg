// Purpose: the cost of a function call, with n from the command line.
public class FibPurpose {
    static long fib(long n) {
        if (n < 2) return n;
        return fib(n - 1) + fib(n - 2);
    }

    public static void main(String[] args) {
        long n = args.length > 0 ? Long.parseLong(args[0]) : 35;
        System.out.println(fib(n));
    }
}
