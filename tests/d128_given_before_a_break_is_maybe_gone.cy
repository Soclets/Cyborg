// The loop might not run at all, and it's dead on the path where it did.
// trace: D128, D2
fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [] -> void:
    let $s: str = "a";
    let $n: i32 = 1;
    for $i in range[from: 1, to: $n]:
        eat(Give $s);
        break;
    end for
    eat(Give $s);
    return void;
end fn.main
