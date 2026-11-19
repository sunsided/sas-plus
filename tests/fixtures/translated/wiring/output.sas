begin_version
3
end_version
begin_metric
1
end_metric
9
begin_variable
var0
0
2
Atom powered(plant)
NegatedAtom powered(plant)
end_variable
begin_variable
var1
-1
2
Atom cable(plant, shed)
NegatedAtom cable(plant, shed)
end_variable
begin_variable
var2
0
2
Atom powered(shed)
NegatedAtom powered(shed)
end_variable
begin_variable
var3
-1
2
Atom cable(plant, junction)
NegatedAtom cable(plant, junction)
end_variable
begin_variable
var4
0
2
Atom powered(junction)
NegatedAtom powered(junction)
end_variable
begin_variable
var5
-1
2
Atom cable(junction, lamp)
NegatedAtom cable(junction, lamp)
end_variable
begin_variable
var6
-1
2
Atom cable(plant, lamp)
NegatedAtom cable(plant, lamp)
end_variable
begin_variable
var7
-1
2
Atom cable(shed, lamp)
NegatedAtom cable(shed, lamp)
end_variable
begin_variable
var8
0
2
Atom powered(lamp)
NegatedAtom powered(lamp)
end_variable
0
begin_state
1
1
1
1
1
1
1
1
1
end_state
begin_goal
1
8 0
end_goal
5
begin_operator
lay-cable junction lamp
1
8 1
1
0 5 1 0
2
end_operator
begin_operator
lay-cable plant junction
1
4 1
1
0 3 1 0
2
end_operator
begin_operator
lay-cable plant lamp
1
8 1
1
0 6 1 0
5
end_operator
begin_operator
lay-cable plant shed
1
2 1
1
0 1 1 0
1
end_operator
begin_operator
lay-cable shed lamp
1
8 1
1
0 7 1 0
4
end_operator
6
begin_rule
0
0 1 0
end_rule
begin_rule
2
5 0
4 0
8 1 0
end_rule
begin_rule
2
3 0
0 0
4 1 0
end_rule
begin_rule
2
6 0
0 0
8 1 0
end_rule
begin_rule
2
1 0
0 0
2 1 0
end_rule
begin_rule
2
7 0
2 0
8 1 0
end_rule
