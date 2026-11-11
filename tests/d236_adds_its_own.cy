// A struct defines `+`, `x` and a `div` narrowing for itself, keeping their usual precedence.
// trace: D236, D237, D116
import std;
import os;

struct money:
    cents: i64;
end struct money

op + [$a: money, $b: money] returns money:
    return money[cents set_to $a.cents + $b.cents];
end op +

op x [$a: money, $b: i64] returns money:
    return money[cents set_to $a.cents x $b];
end op x

op div.floor [$a: money, $b: i64] returns money:
    return money[cents set_to $a.cents div.floor $b];
end op div.floor

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $price: money set_to money[cents set_to 250];
    let $fee: money set_to money[cents set_to 30];
    let $count: i64 set_to 3;
    let $total: money set_to $price + $fee x $count;
    let $half: money set_to $total div.floor 2;
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $total.cents, $half.cents);
    return void;
end fn.main
