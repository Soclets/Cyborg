// trace: D120
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", first());
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $later);
    return void;
end fn.main

fn first [none] -> i32:
    return second() + 1;
end fn first

fn second [none] -> i32:
    return 41;
end fn second

const $later: i32 = $earlier + 1;
const $earlier: i32 = 1;
