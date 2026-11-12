// An OS runner owns threads, so it's `must_Give`, and leaving without `os::done` doesn't compile.
// trace: D392
import std;
import os;

fn.main [none] -> void:
    let $pool: os::threads = os::threads(count: 2);
    return void;
end fn.main
