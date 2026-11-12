// An atomic is an integer or `bool` that tasks change without a lock, shared with plain `Lend`
// and changed through `.%`. `.%add` is checked like `+`, with `.wrap` and `.saturate`, and every
// operation takes an ordering narrowing. `.%swap` gives back the old value, and
// `.%compare_swap` fails with `not_expected`, carrying what's there now.
// trace: D354, D360, D361
import std;
import os;
import @concur;

fn count [$hits: Lent @concur::atomic<u64>] -> i64:
    $hits.%add(val: 1);
    $hits.%add.relaxed(val: 2);
    return 0;
end fn count

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $runner: std::run::one_thread = std::run::one_thread();
    let $heap: os::heap = os::heap();
    let $hits: @concur::atomic<u64> = @concur::atomic.new(0);
    let $a: @concur::task<i64> = @concur::spawn(on: Lend $runner, alloc: Lend $heap, stack: 4 kib) count(Lend $hits);
    let $b: @concur::task<i64> = @concur::spawn(on: Lend $runner, alloc: Lend $heap, stack: 4 kib) count(Lend $hits);
    let $done: i64 = @concur::task.done(Give $a) + @concur::task.done(Give $b);
    let $now: u64 = $hits.%load();
    let $seen: u64 = $hits.%load.acquire();
    let $small: @concur::atomic<u8> = @concur::atomic.new(250);
    $small.%add.saturate(val: 10);
    let $full: u8 = $small.%load();
    $small.%add.wrap(val: 10);
    let $wrapped: u8 = $small.%load();
    $small.%and.bit(val: 7);
    let $masked: u8 = $small.%load();
    let $old: u8 = $small.%swap(val: 1);
    let $owner: @concur::atomic<i32> = @concur::atomic.new(0);
    $owner.%compare_swap(expect: 0, val: 5) otherwise:
        case not_expected $f:
            std::io::fmt_print(Lend.mut $out, "first lost to {}{newln}", $f.val);
    end otherwise
    $owner.%compare_swap(expect: 0, val: 6) otherwise:
        case not_expected $f:
            std::io::fmt_print(Lend.mut $out, "second lost to {}{newln}", $f.val);
    end otherwise
    std::io::fmt_print(Lend.mut $out, "{} {} {} {} {} {} {} {}{newln}", $done, $now, $seen, $full, $wrapped, $masked, $old, $owner.%load());
    return void;
end fn.main
