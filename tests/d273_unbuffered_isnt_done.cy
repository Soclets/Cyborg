// `os::done` is for a buffered writer, the only `must_Give` one.
// trace: D273
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    os::done(Give $out);
    return void;
end fn.main
