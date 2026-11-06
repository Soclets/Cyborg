// A real program: sort an array in place, with positions and `while`.
// trace: D125, D126, D127, D128, D133
import std;

fn sort [$xs: Lent.mut std::array<i32>[len: 6 items]] -> void:
    let mut $swapped: bool = true;
    while $swapped:
        change $swapped = false;
        for $i in range[from: 1, before: 6]:
            if $xs[pos: $i] > $xs[pos: $i + 1]:
                let $held: i32 = $xs[pos: $i];
                change $xs[pos: $i] = $xs[pos: $i + 1];
                change $xs[pos: $i + 1] = $held;
                change $swapped = true;
            end if
        end for
    end while
    return void;
end fn sort

fn.main [] -> void:
    let mut $xs: std::array<i32>[len: 6 items] = [5, 2, 9, 1, 7, 3];
    sort(Lend.mut $xs);
    for $x in $xs:
        std::println($x as str);
    end for
    return void;
end fn.main
