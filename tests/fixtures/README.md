# output.sas fixtures

Tasks in Fast Downward's translator output format (version 3), as specified in
`src/translate/fast_downward/translate/sas_tasks.py` of Fast Downward and at
<https://www.fast-downward.org/latest/documentation/translator-output-format/>.

These files were **hand-encoded** to match that writer exactly. They are not
translator output. `gripper-costs` and `gripper-unit` differ only in the metric
line: with metric `0`, every operator costs 1 regardless of the cost field.

| Fixture | Features | Optimal cost |
|---|---|---|
| `gripper-costs` | mutex groups, action costs (move 3, pick/drop 1) | 7 |
| `gripper-unit` | same task, `metric 0` | 5 |
| `conditional-effects` | conditional effects, `<none of those>` value | not checked |
| `derived-predicates` | derived variable (axiom layer 0), one axiom rule | 2 |

Real translator output lives in [`translated/`](translated). To add more:

```bash
fast-downward.py --translate domain.pddl problem.pddl   # writes output.sas
```
