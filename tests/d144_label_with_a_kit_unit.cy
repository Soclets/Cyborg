// trace: D144, D112
import std;
import os;

fn wait [$t: i32[unit: std::units::time.seconds.span]] returns void:
    labels [$t as for.std::units::time.seconds];
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();

    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($t as i32));
    return void;
end fn wait

fn.main [none] returns void:
    wait(for.std::units::time.seconds: 2);
    return void;
end fn.main
