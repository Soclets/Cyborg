// `os::events(count:, alloc:)` makes an event runner, its `alloc:` for its queue of waiting
// tasks, and a spawn onto it names each task's stack, as one onto `std::run::one_thread` does.
// It owns threads, so it ends with `os::done`, as `os::threads` does.
// trace: D392, D350, D351, D352, D359
import std;
import os;
import @concur;

fn square [$n: i64] -> i64:
    return $n x $n;
end fn square

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $events: os::events = os::events(count: 1, alloc: Lend $heap);
    defer os::done(Give $events);
    let $pool: os::threads = os::threads(count: max);
    defer os::done(Give $pool);
    let $a: @concur::task<i64> = @concur::spawn(on: Lend $events, alloc: Lend $heap, stack: 16 kib) square(6);
    let $b: @concur::task<i64> = @concur::spawn(on: Lend $pool) square(7);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", @concur::task.done(Give $a) + @concur::task.done(Give $b));
    return void;
end fn.main
