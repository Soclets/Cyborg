// Every spawn names its runner.
// trace: D345
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
    let $xs: std::coll::list<i64>[len.max: 4 items] = [1];
    let $t: @concur::task<i64> = @concur::spawn() read_first(Lend $xs);
    let $first: i64 = @concur::task.done(Give $t);
    return void;
end fn.main
