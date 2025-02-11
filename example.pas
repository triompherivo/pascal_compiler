{ A complex test program for our Pascal compiler/interpreter }
var
    n: integer;
    x: real;
    s: string;
    p: record;
           name: string;
           age: integer;
       end;
    i: integer;
begin
    { Read an integer value from input }
    n := readln();
    
    { Compute and print the factorial of n }
    writeln("Factorial:");
    writeln(fact(n));
    
    { Use built-in math functions on x }
    x := 0.5;
    writeln("sin(x):");
    writeln(sin(x));
    writeln("cos(x):");
    writeln(cos(x));
    
    { Use a built-in string function }
    s := "hello world";
    writeln("Uppercase:");
    writeln(uppercase(s));
    
    { Create a record literal and access its fields }
    p := record( name: "John Doe"; age: 30 );
    writeln("Record p.name:");
    writeln(p.name);
    writeln("Record p.age:");
    writeln(p.age);
    
    { For loop: iterate from 1 to n }
    for i := 1 to n do begin
        writeln("Iteration ", i);
    end;
    
    { Case statement on n }
    case n of
         0: writeln("Zero");
         1: writeln("One");
         else writeln("Other");
    end;
    
    { Repeat-until loop: count down from n to 0 }
    repeat
        writeln("Counting down: ", n);
        n := n - 1
    until n = 0;
    
    { File I/O demonstration:
      Open (or create) "output.txt" for writing and write a message to it.
      (For simplicity, we call fopen and fwrite inline.) }
    writeln("File I/O demonstration:");
    writeln("Writing to file...");
    fwrite(fopen("output.txt", "w"), "Complex program complete.");
end.
.
