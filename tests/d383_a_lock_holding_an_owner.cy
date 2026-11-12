// A lock holding a `must_Give` value is `must_Give` too.
// trace: D383
import std;
import os;
import @concur;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $list: @concur::lock<std::coll::list<i64>[len.max: inf items]> = @concur::lock.new(std::coll::list.from([1, 2], alloc: Lend $heap));
    return void;
end fn.main
