// Ceil: as fast as Java can go. The sum of 1 to n is n x (n + 1) / 2.
public class LoopCeil {
    public static void main(String[] args) {
        long n = 100_000_000L;
        System.out.println(n * (n + 1) / 2);
    }
}
