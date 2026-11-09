// trace: D120
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", first());
    std::io::fmt_print("{}{newln}", $later);
    return void;
end fn.main

fn first [] -> i32:
    return second() + 1;
end fn first

fn second [] -> i32:
    return 41;
end fn second

const $later: i32 = $earlier + 1;
const $earlier: i32 = 1;
