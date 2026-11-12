// A list that grows on the heap is filled from items with `std::coll::list.from`, and heap
// text with `str.from`, which takes a string as well as a number.
// trace: D369, D321, D328
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = std::coll::list.from([1, 2, 3], alloc: Lend $heap);
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    let $name: str[len.max: 16 bytes] = "kitchen";
    let $view: str = "hall";
    let $a: str[len.max: inf bytes] = str.from("abc", alloc: Lend $heap);
    defer str.done(Give $a, alloc: Lend $heap);
    let $b: str[len.max: inf bytes] = str.from($name, alloc: Lend $heap);
    defer str.done(Give $b, alloc: Lend $heap);
    let $c: str[len.max: inf bytes] = str.from($view, alloc: Lend $heap);
    defer str.done(Give $c, alloc: Lend $heap);
    std::io::fmt_print(Lend.mut $out, "{} {} {} {} {}{newln}", $xs.len, $xs[pos: 4], $a, $b, $c);
    return void;
end fn.main
