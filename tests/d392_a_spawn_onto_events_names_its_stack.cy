// A spawn onto an event runner takes `alloc:` and `stack:` for its task's own stack.
// trace: D392, D351, D352
import std;
import os;
import @concur;

fn one [none] -> i64:
    return 1;
end fn one

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $events: os::events = os::events(count: max, alloc: Lend $heap);
    defer os::done(Give $events);
    let $t: @concur::task<i64> = @concur::spawn(on: Lend $events) one();
    let $first: i64 = @concur::task.done(Give $t);
    return void;
end fn.main
