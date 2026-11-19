begin_version
3
end_version
begin_metric
1
end_metric
9
begin_variable
var0
-1
3
Atom van-at(depot)
Atom van-at(north)
Atom van-at(south)
end_variable
begin_variable
var1
-1
2
Atom at(parcel2, north)
NegatedAtom at(parcel2, north)
end_variable
begin_variable
var2
-1
2
Atom at(parcel2, south)
NegatedAtom at(parcel2, south)
end_variable
begin_variable
var3
-1
2
Atom at(parcel2, depot)
NegatedAtom at(parcel2, depot)
end_variable
begin_variable
var4
-1
2
Atom loaded(parcel2)
NegatedAtom loaded(parcel2)
end_variable
begin_variable
var5
-1
2
Atom at(parcel1, depot)
NegatedAtom at(parcel1, depot)
end_variable
begin_variable
var6
-1
2
Atom at(parcel1, north)
NegatedAtom at(parcel1, north)
end_variable
begin_variable
var7
-1
2
Atom at(parcel1, south)
NegatedAtom at(parcel1, south)
end_variable
begin_variable
var8
-1
2
Atom loaded(parcel1)
NegatedAtom loaded(parcel1)
end_variable
0
begin_state
0
0
1
1
1
0
1
1
1
end_state
begin_goal
4
3 0
4 1
7 0
8 1
end_goal
18
begin_operator
drive depot north
0
5
1 8 0 5 -1 1
1 8 0 6 -1 0
1 4 0 3 -1 1
1 4 0 1 -1 0
0 0 0 1
4
end_operator
begin_operator
drive depot south
0
5
1 8 0 5 -1 1
1 8 0 7 -1 0
1 4 0 3 -1 1
1 4 0 2 -1 0
0 0 0 2
7
end_operator
begin_operator
drive north depot
0
5
1 8 0 5 -1 0
1 8 0 6 -1 1
1 4 0 3 -1 0
1 4 0 1 -1 1
0 0 1 0
4
end_operator
begin_operator
drive north south
0
5
1 8 0 6 -1 1
1 8 0 7 -1 0
1 4 0 1 -1 1
1 4 0 2 -1 0
0 0 1 2
2
end_operator
begin_operator
drive south depot
0
5
1 8 0 5 -1 0
1 8 0 7 -1 1
1 4 0 3 -1 0
1 4 0 2 -1 1
0 0 2 0
7
end_operator
begin_operator
drive south north
0
5
1 8 0 6 -1 0
1 8 0 7 -1 1
1 4 0 1 -1 0
1 4 0 2 -1 1
0 0 2 1
2
end_operator
begin_operator
load parcel1 depot
2
5 0
0 0
1
0 8 1 0
1
end_operator
begin_operator
load parcel1 north
2
6 0
0 1
1
0 8 1 0
1
end_operator
begin_operator
load parcel1 south
2
7 0
0 2
1
0 8 1 0
1
end_operator
begin_operator
load parcel2 depot
2
3 0
0 0
1
0 4 1 0
1
end_operator
begin_operator
load parcel2 north
2
1 0
0 1
1
0 4 1 0
1
end_operator
begin_operator
load parcel2 south
2
2 0
0 2
1
0 4 1 0
1
end_operator
begin_operator
unload parcel1 depot
1
0 0
1
0 8 0 1
1
end_operator
begin_operator
unload parcel1 north
1
0 1
1
0 8 0 1
1
end_operator
begin_operator
unload parcel1 south
1
0 2
1
0 8 0 1
1
end_operator
begin_operator
unload parcel2 depot
1
0 0
1
0 4 0 1
1
end_operator
begin_operator
unload parcel2 north
1
0 1
1
0 4 0 1
1
end_operator
begin_operator
unload parcel2 south
1
0 2
1
0 4 0 1
1
end_operator
0
