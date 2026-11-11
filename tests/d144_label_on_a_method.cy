// trace: D144, D130
import std;
import os;

fam.unit screen:
    screen.pixels: base;
end fam.unit screen

struct cursor:
    x: i32[unit: screen.pixels];
end struct cursor

fn cursor.%%move [$c: Lent.mut cursor, $by: i32[unit: screen.pixels]] returns void:
    labels [$by as by.screen.pixels];

    change $c.x set_to $c.x + $by;
    return void;
end fn cursor.%%move

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $c: cursor set_to cursor[x set_to 1 screen.pixels];
    $c.%%move(by.screen.pixels: 2);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($c.x as i32));
    return void;
end fn.main
