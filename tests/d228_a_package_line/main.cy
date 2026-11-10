// A file doesn't say which package it belongs to, so Go's `package main` isn't a fact.
// trace: D228
import std;

fn.main [none] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
