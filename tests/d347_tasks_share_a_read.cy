// Several tasks can read one list at once.
// trace: D347
import std;
import os;
import @concur;

fn add_one [$xs: std::coll::list.mut<i64>] -> i64:
    change $xs[pos: 1] = $xs[pos: 1] + 1;
    return $xs[pos: 1];
end fn add_one

fn read_first [$xs: std::coll::list<i64>] -> i64:
    return $xs[pos: 1];
end fn read_first

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $runner: std::run::one_thread = std::run::one_thread();
    let $heap: os::heap = os::heap();
    let $xs: std::coll::list<i64>[len.max: 4 items] = [7];
    let $a: @concur::task<i64> = @concur::spawn(on: Lend $runner, alloc: Lend $heap, stack: 4 kib) read_first(Lend $xs);
    let $b: @concur::task<i64> = @concur::spawn(on: Lend $runner, alloc: Lend $heap, stack: 4 kib) read_first(Lend $xs);
    let $sum: i64 = @concur::task.done(Give $a) + @concur::task.done(Give $b);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $sum);
    return void;
end fn.main
