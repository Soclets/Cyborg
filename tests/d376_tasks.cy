// A call becomes a task with `@concur::spawn(on: Lend $runner, alloc: Lend $heap, stack: 4 kib)` in front. Its arguments are taken
// there, and its body runs on the one thread when the code that spawned it first waits, at
// `@concur::task.done`, which gives back what it made and passes its failure up.
// trace: D343, D345, D346, D356, D357, D375, D376, D377
import std;
import os;
import @concur;

fn sum [$xs: std::coll::list<i64>, $out: Lent.mut os::stdout.unbuffered] -> i64:
    let mut $total: i64 = 0;
    for $x in $xs:
        change $total = $total + $x;
    end for
    std::io::fmt_print(Lend.mut $out, "summed {}{newln}", $total);
    return $total;
end fn sum

fn half [$n: i64] -> i64 fails [odd]:
    if $n mod.trunc 2 == 1:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn both [none] -> i64 fails [odd]:
    let $runner: std::run::one_thread = std::run::one_thread();
    let $heap: os::heap = os::heap();
    let $t: @concur::task<i64> = @concur::spawn(on: Lend $runner, alloc: Lend $heap, stack: 4 kib) half(7);
    return @concur::task.done(Give $t)?;
end fn both

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $log: os::stdout.unbuffered = os::stdout.unbuffered();
    let $runner: std::run::one_thread = std::run::one_thread();
    let $heap: os::heap = os::heap();
    let $pool: os::threads = os::threads(count: 4);
    let $xs: std::coll::list<i64>[len.max: 8 items] = [1, 2, 3];
    let $ys: std::coll::list<i64>[len.max: 8 items] = [10, 20];
    let $left: @concur::task<i64> = @concur::spawn(on: Lend $runner, alloc: Lend $heap, stack: 4 kib) sum(Lend $xs, Lend.mut $log);
    std::io::fmt_print(Lend.mut $out, "spawned{newln}");
    let $h: @concur::task<i64> = @concur::spawn(on: Lend $pool) half(10);
    let $whole: i64 = @concur::task.done(Give $left);
    let $part: i64 = @concur::task.done(Give $h) otherwise:
        case odd:
            instead 0;
    end otherwise
    let $total: i64 = $whole + $part;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $total);
    let $odd: i64 = both() otherwise:
        case odd:
            instead -1;
    end otherwise
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $odd, $ys.len);
    return void;
end fn.main
