// A generic call's result has the type its arguments fill, so it can go into maths or into
// another generic call, and an argument like `17 div.trunc 18` takes the type the others fill.
// trace: D241
import std;
import os;

interface addable<?self>:
    op + [$a: ?self, $b: ?self] -> ?self;
end interface addable

type_set whole:
    i32;
    i64;
end type_set whole

i32 satisfies addable;

fn gsum<?item: whole and addable> [$a: ?item, $b: ?item] -> ?item:
    return $a + $b;
end fn gsum

fn gtwice<?item: addable> [$a: Lent ?item] -> ?item:
    return $a + $a;
end fn gtwice

fn gkeep<?item: any.type> [$a: Given ?item] -> ?item:
    let $b: ?item = Give $a;
    return Give $b;
end fn gkeep

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 20;
    let $b: i32 = gkeep($a) div.trunc 3;
    let $c: i32 = gsum($a, gtwice($a));
    let $d: i32 = gsum($a, 17 div.trunc 2);
    let $e: i32 = gsum($a, $a xor.bit 3);
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $b, $c, $d, $e);
    return void;
end fn.main
