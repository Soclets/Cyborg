// By D116's table this is `(not $a) < 3`, so it's refused as a type mistake.
// Reported to the design session, because a reader would take it as `not ($a < 3)`.
// trace: D116
fn.main [] -> void:
    let $a: i32 = 1;
    if not $a < 3:
        return void;
    end if
    return void;
end fn.main
