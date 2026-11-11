// Purpose: a loop with an add, with the count from the command line.
public class LoopPurpose {
    public static void main(String[] args) {
        long n = args.length > 0 ? Long.parseLong(args[0]) : 100_000_000L;
        long sum = 0;
        for (long i = 1; i <= n; i++) sum += i;
        System.out.println(sum);
    }
}
