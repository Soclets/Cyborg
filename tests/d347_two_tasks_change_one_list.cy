// `Lend.mut` goes to one task only, so a second task changing the same list ends the first's
// borrow, and ending the first task after that is refused.
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
    let $runner: std::run::one_thread = std::run::one_thread();
    let mut $xs: std::coll::list<i64>[len.max: 4 items] = [1];
    let $a: @concur::task<i64> = @concur::spawn(on: Lend $runner) add_one(Lend.mut $xs);
    let $b: @concur::task<i64> = @concur::spawn(on: Lend $runner) add_one(Lend.mut $xs);
    let $first: i64 = @concur::task.done(Give $a);
    let $second: i64 = @concur::task.done(Give $b);
    return void;
end fn.main
