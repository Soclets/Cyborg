// A buffered writer's size says its unit.
// trace: D273
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024);
    os::done(Give $out);
    return void;
end fn.main
