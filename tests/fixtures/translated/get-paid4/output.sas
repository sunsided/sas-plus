begin_version
3
end_version
begin_metric
0
end_metric
20
begin_variable
var0
-1
2
Atom at(b, b)
NegatedAtom at(b, b)
end_variable
begin_variable
var1
-1
2
Atom at(b, bank)
NegatedAtom at(b, bank)
end_variable
begin_variable
var2
-1
2
Atom at(b, d)
NegatedAtom at(b, d)
end_variable
begin_variable
var3
-1
2
Atom at(b, office)
NegatedAtom at(b, office)
end_variable
begin_variable
var4
-1
2
Atom at(b, p)
NegatedAtom at(b, p)
end_variable
begin_variable
var5
-1
2
Atom at(b, home)
NegatedAtom at(b, home)
end_variable
begin_variable
var6
-1
2
Atom in(p)
NegatedAtom in(p)
end_variable
begin_variable
var7
-1
2
Atom at(p, b)
NegatedAtom at(p, b)
end_variable
begin_variable
var8
-1
2
Atom at(p, d)
NegatedAtom at(p, d)
end_variable
begin_variable
var9
-1
2
Atom at(p, home)
NegatedAtom at(p, home)
end_variable
begin_variable
var10
-1
2
Atom at(p, office)
NegatedAtom at(p, office)
end_variable
begin_variable
var11
-1
2
Atom at(p, p)
NegatedAtom at(p, p)
end_variable
begin_variable
var12
-1
2
Atom at(p, bank)
NegatedAtom at(p, bank)
end_variable
begin_variable
var13
-1
2
Atom in(d)
NegatedAtom in(d)
end_variable
begin_variable
var14
-1
2
Atom at(d, b)
NegatedAtom at(d, b)
end_variable
begin_variable
var15
-1
2
Atom at(d, bank)
NegatedAtom at(d, bank)
end_variable
begin_variable
var16
-1
2
Atom at(d, d)
NegatedAtom at(d, d)
end_variable
begin_variable
var17
-1
2
Atom at(d, home)
NegatedAtom at(d, home)
end_variable
begin_variable
var18
-1
2
Atom at(d, p)
NegatedAtom at(d, p)
end_variable
begin_variable
var19
-1
2
Atom at(d, office)
NegatedAtom at(d, office)
end_variable
0
begin_state
1
1
1
1
1
0
0
1
1
0
1
1
1
1
1
1
1
0
1
1
end_state
begin_goal
3
5 0
12 0
19 0
end_goal
44
begin_operator
mov-b b bank
0
6
0 0 0 1
0 1 -1 0
1 13 0 14 -1 1
1 13 0 15 -1 0
1 6 0 7 -1 1
1 6 0 12 -1 0
1
end_operator
begin_operator
mov-b b d
0
6
0 0 0 1
0 2 -1 0
1 13 0 14 -1 1
1 13 0 16 -1 0
1 6 0 7 -1 1
1 6 0 8 -1 0
1
end_operator
begin_operator
mov-b b home
0
6
0 0 0 1
0 5 -1 0
1 13 0 14 -1 1
1 13 0 17 -1 0
1 6 0 7 -1 1
1 6 0 9 -1 0
1
end_operator
begin_operator
mov-b b office
0
6
0 0 0 1
0 3 -1 0
1 13 0 14 -1 1
1 13 0 19 -1 0
1 6 0 7 -1 1
1 6 0 10 -1 0
1
end_operator
begin_operator
mov-b b p
0
6
0 0 0 1
0 4 -1 0
1 13 0 14 -1 1
1 13 0 18 -1 0
1 6 0 7 -1 1
1 6 0 11 -1 0
1
end_operator
begin_operator
mov-b bank b
0
6
0 0 -1 0
0 1 0 1
1 13 0 14 -1 0
1 13 0 15 -1 1
1 6 0 7 -1 0
1 6 0 12 -1 1
1
end_operator
begin_operator
mov-b bank d
0
6
0 1 0 1
0 2 -1 0
1 13 0 15 -1 1
1 13 0 16 -1 0
1 6 0 12 -1 1
1 6 0 8 -1 0
1
end_operator
begin_operator
mov-b bank home
0
6
0 1 0 1
0 5 -1 0
1 13 0 15 -1 1
1 13 0 17 -1 0
1 6 0 12 -1 1
1 6 0 9 -1 0
1
end_operator
begin_operator
mov-b bank office
0
6
0 1 0 1
0 3 -1 0
1 13 0 15 -1 1
1 13 0 19 -1 0
1 6 0 12 -1 1
1 6 0 10 -1 0
1
end_operator
begin_operator
mov-b bank p
0
6
0 1 0 1
0 4 -1 0
1 13 0 15 -1 1
1 13 0 18 -1 0
1 6 0 12 -1 1
1 6 0 11 -1 0
1
end_operator
begin_operator
mov-b d b
0
6
0 0 -1 0
0 2 0 1
1 13 0 14 -1 0
1 13 0 16 -1 1
1 6 0 7 -1 0
1 6 0 8 -1 1
1
end_operator
begin_operator
mov-b d bank
0
6
0 1 -1 0
0 2 0 1
1 13 0 15 -1 0
1 13 0 16 -1 1
1 6 0 12 -1 0
1 6 0 8 -1 1
1
end_operator
begin_operator
mov-b d home
0
6
0 2 0 1
0 5 -1 0
1 13 0 16 -1 1
1 13 0 17 -1 0
1 6 0 8 -1 1
1 6 0 9 -1 0
1
end_operator
begin_operator
mov-b d office
0
6
0 2 0 1
0 3 -1 0
1 13 0 16 -1 1
1 13 0 19 -1 0
1 6 0 8 -1 1
1 6 0 10 -1 0
1
end_operator
begin_operator
mov-b d p
0
6
0 2 0 1
0 4 -1 0
1 13 0 16 -1 1
1 13 0 18 -1 0
1 6 0 8 -1 1
1 6 0 11 -1 0
1
end_operator
begin_operator
mov-b home b
0
6
0 0 -1 0
0 5 0 1
1 13 0 14 -1 0
1 13 0 17 -1 1
1 6 0 7 -1 0
1 6 0 9 -1 1
1
end_operator
begin_operator
mov-b home bank
0
6
0 1 -1 0
0 5 0 1
1 13 0 15 -1 0
1 13 0 17 -1 1
1 6 0 12 -1 0
1 6 0 9 -1 1
1
end_operator
begin_operator
mov-b home d
0
6
0 2 -1 0
0 5 0 1
1 13 0 16 -1 0
1 13 0 17 -1 1
1 6 0 8 -1 0
1 6 0 9 -1 1
1
end_operator
begin_operator
mov-b home office
0
6
0 5 0 1
0 3 -1 0
1 13 0 17 -1 1
1 13 0 19 -1 0
1 6 0 9 -1 1
1 6 0 10 -1 0
1
end_operator
begin_operator
mov-b home p
0
6
0 5 0 1
0 4 -1 0
1 13 0 17 -1 1
1 13 0 18 -1 0
1 6 0 9 -1 1
1 6 0 11 -1 0
1
end_operator
begin_operator
mov-b office b
0
6
0 0 -1 0
0 3 0 1
1 13 0 14 -1 0
1 13 0 19 -1 1
1 6 0 7 -1 0
1 6 0 10 -1 1
1
end_operator
begin_operator
mov-b office bank
0
6
0 1 -1 0
0 3 0 1
1 13 0 15 -1 0
1 13 0 19 -1 1
1 6 0 12 -1 0
1 6 0 10 -1 1
1
end_operator
begin_operator
mov-b office d
0
6
0 2 -1 0
0 3 0 1
1 13 0 16 -1 0
1 13 0 19 -1 1
1 6 0 8 -1 0
1 6 0 10 -1 1
1
end_operator
begin_operator
mov-b office home
0
6
0 5 -1 0
0 3 0 1
1 13 0 17 -1 0
1 13 0 19 -1 1
1 6 0 9 -1 0
1 6 0 10 -1 1
1
end_operator
begin_operator
mov-b office p
0
6
0 3 0 1
0 4 -1 0
1 13 0 19 -1 1
1 13 0 18 -1 0
1 6 0 10 -1 1
1 6 0 11 -1 0
1
end_operator
begin_operator
mov-b p b
0
6
0 0 -1 0
0 4 0 1
1 13 0 14 -1 0
1 13 0 18 -1 1
1 6 0 7 -1 0
1 6 0 11 -1 1
1
end_operator
begin_operator
mov-b p bank
0
6
0 1 -1 0
0 4 0 1
1 13 0 15 -1 0
1 13 0 18 -1 1
1 6 0 12 -1 0
1 6 0 11 -1 1
1
end_operator
begin_operator
mov-b p d
0
6
0 2 -1 0
0 4 0 1
1 13 0 16 -1 0
1 13 0 18 -1 1
1 6 0 8 -1 0
1 6 0 11 -1 1
1
end_operator
begin_operator
mov-b p home
0
6
0 5 -1 0
0 4 0 1
1 13 0 17 -1 0
1 13 0 18 -1 1
1 6 0 9 -1 0
1 6 0 11 -1 1
1
end_operator
begin_operator
mov-b p office
0
6
0 3 -1 0
0 4 0 1
1 13 0 19 -1 0
1 13 0 18 -1 1
1 6 0 10 -1 0
1 6 0 11 -1 1
1
end_operator
begin_operator
put-in d b
0
1
2 0 0 14 0 13 -1 0
1
end_operator
begin_operator
put-in d bank
0
1
2 1 0 15 0 13 -1 0
1
end_operator
begin_operator
put-in d d
0
1
2 2 0 16 0 13 -1 0
1
end_operator
begin_operator
put-in d home
0
1
2 5 0 17 0 13 -1 0
1
end_operator
begin_operator
put-in d office
0
1
2 3 0 19 0 13 -1 0
1
end_operator
begin_operator
put-in d p
0
1
2 4 0 18 0 13 -1 0
1
end_operator
begin_operator
put-in p b
0
1
2 0 0 7 0 6 -1 0
1
end_operator
begin_operator
put-in p bank
0
1
2 1 0 12 0 6 -1 0
1
end_operator
begin_operator
put-in p d
0
1
2 2 0 8 0 6 -1 0
1
end_operator
begin_operator
put-in p home
0
1
2 5 0 9 0 6 -1 0
1
end_operator
begin_operator
put-in p office
0
1
2 3 0 10 0 6 -1 0
1
end_operator
begin_operator
put-in p p
0
1
2 4 0 11 0 6 -1 0
1
end_operator
begin_operator
take-out d
0
1
0 13 -1 1
1
end_operator
begin_operator
take-out p
0
1
0 6 -1 1
1
end_operator
0
