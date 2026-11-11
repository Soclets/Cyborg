// trace: D170

struct reading:
    celsius: i32;
end struct reading

fn warmer [$a: reading, $b: reading] returns Lend from <$a, $b> reading:
    if $a.celsius above $b.celsius:
        return Lend $a;
    end if
    return Lend $b;
end fn warmer

fn.main [none] returns void:
    let $x: reading set_to reading[celsius set_to 21];
    let $y: reading set_to reading[celsius set_to 25];
    let $w: Lend from $x reading set_to warmer(Lend $x, Lend $y);
    return void;
end fn.main
