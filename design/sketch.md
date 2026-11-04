# Sketch

The owner's sketch of Cyborg, likely not final. This is a placeholder, not a specification, and
`: str` is still undecided (see Open in `DECISIONS.md`).

```
fn total($xs: std::list.i32[max_len: inf]) -> i32:
    let mut $sum: i32 = 0;
    for $x in $xs:
        change $sum = $sum + $x;
    end for
    return $sum;
end fn total

fn.main() -> void:
    let mut $xs: std::list.i32[max_len: inf] = empty;
    $xs.%append(val: 4);
    $xs.%append(val: 6);
    $xs.%append(val: 5);

    let $t: i32 = total(Lend $xs);
    std::println("count: ", $xs.len: str);
    if $t at_least 10:
        std::println("big total: ", $t: str);
    end if
end fn.main
```
