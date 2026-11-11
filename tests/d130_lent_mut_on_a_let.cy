// The compiler still checks what `%` leaves unsaid.
// trace: D130, D4, D3

struct box:
    n: i32;
end struct box

fn box.%%bump [$b: Lent.mut box] returns void:
    change $b.n = 2;
    return void;
end fn box.%%bump

fn.main [none] returns void:
    let $b: box = box[n = 1];
    $b.%%bump();
    return void;
end fn.main
