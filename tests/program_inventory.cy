// A real program: stock levels, a report, and a failure handled at the call.
// trace: D2, D9, D13, D20, D45, D57, D115, D120
import std;
import os;

fam.unit stock:
    stock.pieces: base;
end fam.unit stock

union.tagged level:
    plenty;
    low[left: i32[unit: stock.pieces]];
    out;
end union.tagged level

struct line:
    name: str;
    count: i32[unit: stock.pieces];
end struct line

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $bolts: line = line[name = "bolts", count = 40 stock.pieces];
    let $nuts: line = line[name = "nuts", count = 3 stock.pieces];
    let $pins: line = line[name = "pins", count = 0 stock.pieces];
    report(Lend $bolts);
    report(Lend $nuts);
    report(Lend $pins);
    std::io::fmt_print(Lend.mut $out, "pieces: {}{newln}", (total_of(Lend $bolts, Lend $nuts) + $pins.count));

    show_restock(Lend $nuts);
    show_restock(Lend $bolts);
    return void;
end fn.main

fn level_of [$l: line] -> level:
    if $l.count == 0 stock.pieces:
        return level.out;
    end if
    if $l.count below 10 stock.pieces:
        return level.low[left = $l.count];
    end if
    return level.plenty;
end fn level_of

fn report [$l: line] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $level: level = level_of(Lend $l);
    match $level:
        case level.plenty:
            std::io::fmt_print(Lend.mut $out, "{}: plenty{newln}", $l.name);
        case level.low $low:
            std::io::fmt_print(Lend.mut $out, "{}: low, {} left{newln}", $l.name, $low.left);
        case level.out:
            std::io::fmt_print(Lend.mut $out, "{}: out{newln}", $l.name);
    end match
    return void;
end fn report

fn total_of [$a: line, $b: line] -> i32[unit: stock.pieces]:
    return $a.count + $b.count;
end fn total_of

fn order_size [$l: line] -> i32[unit: stock.pieces] fails [nothing_needed]:
    if $l.count at_least 10 stock.pieces:
        fail nothing_needed;
    end if
    return 10 stock.pieces - $l.count;
end fn order_size

fn show_restock [$l: line] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32[unit: stock.pieces] = order_size(Lend $l) otherwise:
        case nothing_needed:
            std::io::fmt_print(Lend.mut $out, "{}: no order{newln}", $l.name);
            return void;
    end otherwise
    std::io::fmt_print(Lend.mut $out, "{}: order {}{newln}", $l.name, $n);
    return void;
end fn show_restock
