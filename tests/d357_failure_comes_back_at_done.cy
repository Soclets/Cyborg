// A task's failure comes back when it's done, so `?` on the spawned call is refused.
// trace: D343, D357, D392
import std;
import os;
import @concur;

fn half [$n: i64] -> i64 fails [odd]:
    if $n mod.trunc 2 == 1:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn both [none] -> i64 fails [odd]:
    let $pool: os::threads = os::threads(count: 2);
    defer os::done(Give $pool);
    let $t: @concur::task<i64> = @concur::spawn(on: Lend $pool) half(7)?;
    return @concur::task.done(Give $t)?;
end fn both

fn.main [none] -> void:
    return void;
end fn.main
