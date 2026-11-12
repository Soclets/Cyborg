// `std::run::one_thread` switches tasks when one waits, so each task has its own stack, and a
// spawn onto it names the allocator and the size.
// trace: D384, D351, D352
import std;
import os;
import @concur;

fn one [none] -> i64:
    return 1;
end fn one

fn.main [none] -> void:
    let $runner: std::run::one_thread = std::run::one_thread();
    let $t: @concur::task<i64> = @concur::spawn(on: Lend $runner) one();
    let $first: i64 = @concur::task.done(Give $t);
    return void;
end fn.main
