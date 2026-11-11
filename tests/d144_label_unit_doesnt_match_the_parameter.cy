// trace: D144

fam.unit screen:
    screen.pixels: base;
    screen.points: base;
end fam.unit screen

fn draw [$x: i32[unit: screen.pixels]] returns void:
    labels [$x as at.screen.points];

    return void;
end fn draw

fn.main [none] returns void:
    return void;
end fn.main
