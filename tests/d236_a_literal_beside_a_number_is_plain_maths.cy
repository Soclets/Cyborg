// A struct's own `div.floor` takes it and an `i64`, but a literal is never a struct, so a
// literal divided by an `i64`, that divided again, and a group of literals divided by an
// `i64`, are plain maths.
// trace: D236
import std;
import os;

struct money:
    cents: i64;
end struct money

op div.floor [$a: Lent money, $b: i64] -> money:
    return money[cents = $a.cents div.floor $b];
end op div.floor

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i64 = 3;
    let $plain: i64 = (100 div.floor $n) div.floor 2;
    let $grouped: i64 = (16 div.trunc 1) div.floor ($n x 2);
    let $price: money = money[cents = 250];
    let $half: money = $price div.floor 2;
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", $plain, $grouped, $half.cents);
    return void;
end fn.main
