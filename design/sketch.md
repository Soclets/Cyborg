# Sketch

The owner's sketch of Cyborg, likely not final. This is a placeholder, not a specification.

```
fn total [$xs: std::list.i32[len.max: inf]] -> i32:
    let mut $sum: i32 = 0;
    for $x in $xs:
        change $sum = $sum + $x;
    end for
    return $sum;
end fn total

fn.main [] -> void:
    let mut $xs: std::list.i32[len.max: inf] = empty;
    $xs.%append(val: 4);
    $xs.%append(val: 6);
    $xs.%append(val: 5);

    let $t: i32 = total(Lend $xs);
    std::println("count: ", $xs.len as str);
    if $t at_least 10:
        std::println("big total: ", $t as str);
    end if
    return void;
end fn.main
```
