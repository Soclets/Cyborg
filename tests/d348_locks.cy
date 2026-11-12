// A lock holds the data it protects, is shared with plain `Lend`, and locking it gives a
// `Lend.mut` of what it holds for one block. One block takes several locks at once, and a lock
// holding an owner ends with `lock.done`, which gives it back.
// trace: D348, D349, D359, D383
import std;
import os;
import @concur;

fn deposit [$account: Lent @concur::lock<i64>, $amount: i64] -> i64:
    @concur::locked $account as $balance:
        change $balance = $balance + $amount;
    end @concur::locked
    return 0;
end fn deposit

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $runner: std::run::one_thread = std::run::one_thread();
    let $from: @concur::lock<i64> = @concur::lock.new(100);
    let $to: @concur::lock<i64> = @concur::lock.new(5);
    let $a: @concur::task<i64> = @concur::spawn(on: Lend $runner, alloc: Lend $heap, stack: 4 kib) deposit(Lend $to, 10);
    let $b: @concur::task<i64> = @concur::spawn(on: Lend $runner, alloc: Lend $heap, stack: 4 kib) deposit(Lend $to, 20);
    let $done: i64 = @concur::task.done(Give $a) + @concur::task.done(Give $b);
    @concur::locked $from, $to as $f, $t:
        change $f = $f minus.wrap 30;
        change $t = $t plus.wrap 30;
        std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $f, $t);
    end @concur::locked
    let $list: @concur::lock<std::coll::list<i64>[len.max: inf items]> = @concur::lock.new(std::coll::list.from([1, 2], alloc: Lend $heap));
    let $back: std::coll::list<i64>[len.max: inf items] = @concur::lock.done(Give $list);
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $done, $back.len);
    std::coll::list.done(Give $back, alloc: Lend $heap);
    return void;
end fn.main
