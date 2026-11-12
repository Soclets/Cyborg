// `@concur` is its own package, imported like any limb.
// trace: D375, D82
import std;

fn one [none] -> i64:
    return 1;
end fn one

fn.main [none] -> void:
    let $runner: std::run::one_thread = std::run::one_thread();
    let $t: @concur::task<i64> = @concur::spawn(on: Lend $runner) one();
    let $first: i64 = @concur::task.done(Give $t);
    return void;
end fn.main
