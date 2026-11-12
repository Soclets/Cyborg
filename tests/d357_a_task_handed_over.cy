// A task handed to another function came through a type that doesn't carry its failures, so
// what its `done` can fail with isn't known.
// trace: D357
import std;
import os;
import @concur;

fn one [none] -> i64:
    return 1;
end fn one

fn finish_it [$t: Given @concur::task<i64>] -> i64:
    return @concur::task.done(Give $t);
end fn finish_it

fn.main [none] -> void:
    let $runner: std::run::one_thread = std::run::one_thread();
    let $heap: os::heap = os::heap();
    let $t: @concur::task<i64> = @concur::spawn(on: Lend $runner, alloc: Lend $heap, stack: 4 kib) one();
    let $first: i64 = finish_it(Give $t);
    return void;
end fn.main
