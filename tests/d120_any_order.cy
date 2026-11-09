// trace: D120
import std;
@config(default);

fn.main [] -> void:
    std::println(first() as str);
    std::println($later as str);
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
