// A read-only list view is passed on bare, like a `str` view, and a `.mut` view takes
// `Lend.mut`, since it can't be copied.
// trace: D381, D379
import std;
import os;

fn first [$xs: std::coll::list<i32>] -> i32:
    return $xs[pos: 1];
end fn first

fn through [$xs: std::coll::list<i32>] -> i32:
    return first($xs);
end fn through

fn bump [$xs: std::coll::list.mut<i32>] -> void:
    change $xs[pos: 1] = $xs[pos: 1] + 1;
    return void;
end fn bump

fn bump_twice [$xs: std::coll::list.mut<i32>] -> void:
    bump(Lend.mut $xs);
    bump(Lend.mut $xs);
    return void;
end fn bump_twice

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $ys: std::coll::list<i32>[len.max: 4 items] = [5];
    bump_twice(Lend.mut $ys);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", through(Lend $ys));
    return void;
end fn.main
