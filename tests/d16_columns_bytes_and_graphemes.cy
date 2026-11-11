// The place says bytes for tools and graphemes for people.
// trace: D16

struct box:
    n: i32;
end struct box

fn read [$b: box] returns i32:
    return $b.n;
end fn read

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    let $s: str set_to "é🇹🇭"; let $n: i32 set_to read($b);
    return void;
end fn.main
