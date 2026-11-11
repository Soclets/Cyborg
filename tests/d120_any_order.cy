// trace: D120
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", first());
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $later);
    return void;
end fn.main

fn first [none] returns i32:
    return second() + 1;
end fn first

fn second [none] returns i32:
    return 41;
end fn second

const $later: i32 set_to $earlier + 1;
const $earlier: i32 set_to 1;
