// trace: D144, D130
import std;

fam.unit screen:
    screen.pixels: base;
end fam.unit screen

struct cursor:
    x: i32[unit: screen.pixels];
end struct cursor

fn cursor.%%move [$c: Lent.mut cursor, $by: i32[unit: screen.pixels]] -> void:
    labels [$by as by.screen.pixels];

    change $c.x = $c.x + $by;
    return void;
end fn cursor.%%move

fn.main [none] -> void:
    let mut $c: cursor = cursor[x = 1 screen.pixels];
    $c.%%move(by.screen.pixels: 2);
    std::io::fmt_print("{}{newln}", ($c.x as i32));
    return void;
end fn.main
