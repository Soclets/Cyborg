// A struct defines `+`, `x` and a `div` narrowing for itself, keeping their usual precedence.
// trace: D236, D237, D116
import std;
import os;

struct money:
    cents: i64;
end struct money

op + [$a: Lent money, $b: Lent money] -> money:
    return money[cents = $a.cents + $b.cents];
end op +

op x [$a: Lent money, $b: i64] -> money:
    return money[cents = $a.cents x $b];
end op x

op div.floor [$a: Lent money, $b: i64] -> money:
    return money[cents = $a.cents div.floor $b];
end op div.floor

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $price: money = money[cents = 250];
    let $fee: money = money[cents = 30];
    let $count: i64 = 3;
    let $total: money = $price + $fee x $count;
    let $half: money = $total div.floor 2;
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $total.cents, $half.cents);
    return void;
end fn.main
