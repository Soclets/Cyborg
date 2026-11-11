// Normal: adding 1 to 100,000,000 in a loop, written plainly.
public class LoopNormal {
    public static void main(String[] args) {
        long sum = 0;
        for (long i = 1; i <= 100_000_000L; i++) sum += i;
        System.out.println(sum);
    }
}
