// An event runner's queue of waiting tasks grows, so `os::events` takes an allocator.
// trace: D392, D321
import std;
import os;

fn.main [none] -> void:
    let $events: os::events = os::events(count: 4);
    defer os::done(Give $events);
    return void;
end fn.main
