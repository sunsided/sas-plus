# Translator-generated fixtures

`output.sas` files produced by Fast Downward's translator
([aibasel/downward](https://github.com/aibasel/downward) at commit `a744e44`), via

```bash
PYTHONPATH=src/translate python3 -m fast_downward.translate domain.pddl problem.pddl --sas-file output.sas
```

Each folder holds the PDDL input next to the output.

| Fixture | Domain | Problem | Variables / operators | Optimal cost |
|---|---|---|---|---|
| `sussman-anomaly` | VHPOP `blocks-world-domain.pddl` | VHPOP `sussman-anomaly.pddl` | 6 / 18 | 3 |
| `gripper-2` | VHPOP `gripper-domain.pddl` | VHPOP `gripper-2.pddl` (goal: ball1 only) | 5 / 18 | 3 |
| `gripper-4` | VHPOP `gripper-domain.pddl` | VHPOP `gripper-4.pddl` (3 of 4 balls) | 7 / 34 | 9 |
| `bw-large-a` | VHPOP `simple-blocks-domain.pddl` | VHPOP `bw-large-a.pddl` | 18 / 648 | not checked |
| `logistics-e` | VHPOP `logistics-domain.pddl` | Potoroo `logistics-e.pddl` | 19 / 756 | not checked |
| `gripper-costs` | own, action costs (move 3, pick/drop 1) | own | 5 / 18 | 7 |
| `courier` | own, action costs **and** conditional effects | own | 9 / 18 | (conditional effects) |
| `get-paid4` | VHPOP `briefcase-world-domain.pddl` (conditional effects) | VHPOP `get-paid4.pddl` | 20 / 44 | (conditional effects) |

`gripper-costs` encodes the same task as the hand-written `../gripper-costs`, and the tests check that both have the same optimal cost.
In `courier` and `get-paid4`, the translator keeps conditional effects on the operators that move a container (van, briefcase) along with its contents. `to_task()` rejects those tasks with `ConversionError::ConditionalEffect`.

Optimal costs are verified by exhaustive search (`Task::perfect_heuristic`) in the tests.

## Provenance and licences

`gripper-costs` and `courier` were written for this repository and fall under its licence (MIT OR Apache-2.0).

All other PDDL files were copied unchanged from the test data of
[sunsided/pddl-rs](https://github.com/sunsided/pddl-rs) (`tests/data/`):

- **VHPOP** files come from the `examples/` folder of [VHPOP](https://github.com/hlsyounes/vhpop) by Håkan L. S. Younes, at commit `cf27db2ed46f8bd64cb78fe256cb5cf3ad8174cf`. They are distributed under the **GNU General Public License, version 2**; see [`COPYING.vhpop`](COPYING.vhpop).
- **`logistics-e/problem.pddl`** comes from the `examples/` folder of [Potoroo](https://github.com/HarrisonOates/potoroo) by Harrison Oates, at commit `1264ed6cefaf3b72233366c44a3a9ac5303b43c2`, under the **Apache License 2.0**; see [`LICENSE.potoroo`](LICENSE.potoroo) and [`NOTICE.potoroo`](NOTICE.potoroo).

Each `output.sas` file is derived from its inputs and falls under the same licences. All of this is test input only. It is not compiled into `sas-plus` and is excluded from its published package.
