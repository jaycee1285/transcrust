| variant | set | arm | settled | correct | wrong-name | wrong-keep | passed | hybrid exact | hybrid repairs | hybrid unwanted |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| v1 | household | single | 12 | 12 | 0 | 0 | 4 | 16/16 | 6/6 | 0/10 |
| v1 | household | two | 16 | 12 | 4 | 0 | 0 | 12/16 | 6/6 | 4/10 |
| v1 | household | both | 28 | 24 | 4 | 0 | 4 | 28/32 | 12/12 | 4/20 |
| v1 | synthetic | single | 17 | 11 | 6 | 0 | 33 | 19/50 | 8/8 | 14/42 |
| v1 | synthetic | two | 23 | 16 | 7 | 0 | 27 | 35/50 | 19/20 | 14/30 |
| v1 | synthetic | both | 40 | 27 | 13 | 0 | 60 | 54/100 | 27/28 | 28/72 |
| v2-local | household | single | 12 | 12 | 0 | 0 | 4 | 16/16 | 6/6 | 0/10 |
| v2-local | household | two | 12 | 12 | 0 | 0 | 4 | 16/16 | 6/6 | 0/10 |
| v2-local | household | both | 24 | 24 | 0 | 0 | 8 | 32/32 | 12/12 | 0/20 |
| v2-local | synthetic | single | 17 | 11 | 6 | 0 | 33 | 19/50 | 8/8 | 14/42 |
| v2-local | synthetic | two | 17 | 13 | 4 | 0 | 33 | 35/50 | 17/20 | 12/30 |
| v2-local | synthetic | both | 34 | 24 | 10 | 0 | 66 | 54/100 | 25/28 | 26/72 |
| v2-gloss | household | single | 12 | 12 | 0 | 0 | 4 | 16/16 | 6/6 | 0/10 |
| v2-gloss | household | two | 16 | 12 | 4 | 0 | 0 | 12/16 | 6/6 | 4/10 |
| v2-gloss | household | both | 28 | 24 | 4 | 0 | 4 | 28/32 | 12/12 | 4/20 |
| v2-gloss | synthetic | single | 17 | 11 | 6 | 0 | 33 | 19/50 | 8/8 | 14/42 |
| v2-gloss | synthetic | two | 23 | 16 | 7 | 0 | 27 | 35/50 | 19/20 | 14/30 |
| v2-gloss | synthetic | both | 40 | 27 | 13 | 0 | 60 | 54/100 | 27/28 | 28/72 |
| v2 | household | single | 12 | 12 | 0 | 0 | 4 | 16/16 | 6/6 | 0/10 |
| v2 | household | two | 12 | 12 | 0 | 0 | 4 | 16/16 | 6/6 | 0/10 |
| v2 | household | both | 24 | 24 | 0 | 0 | 8 | 32/32 | 12/12 | 0/20 |
| v2 | synthetic | single | 17 | 11 | 6 | 0 | 33 | 19/50 | 8/8 | 14/42 |
| v2 | synthetic | two | 17 | 13 | 4 | 0 | 33 | 35/50 | 17/20 | 12/30 |
| v2 | synthetic | both | 34 | 24 | 10 | 0 | 66 | 54/100 | 25/28 | 26/72 |
| v2+recency | household | single | 12 | 12 | 0 | 0 | 4 | 16/16 | 6/6 | 0/10 |
| v2+recency | household | two | 14 | 12 | 2 | 0 | 2 | 14/16 | 6/6 | 2/10 |
| v2+recency | household | both | 26 | 24 | 2 | 0 | 6 | 30/32 | 12/12 | 2/20 |
| v2+recency | synthetic | single | 17 | 11 | 6 | 0 | 33 | 19/50 | 8/8 | 14/42 |
| v2+recency | synthetic | two | 24 | 13 | 11 | 0 | 26 | 33/50 | 17/20 | 14/30 |
| v2+recency | synthetic | both | 41 | 24 | 17 | 0 | 59 | 52/100 | 25/28 | 28/72 |

## Label changes vs v1 (set, arm, v1 -> variant, gold, deciding rule)
- **v2-local**: 10 changed, 0 now match gold
  - household/two: name -> pass (gold keep; None)
  - household/two: name -> pass (gold keep; None)
  - household/two: name -> pass (gold keep; None)
  - household/two: name -> pass (gold keep; None)
  - synthetic/two: name -> pass (gold name; None)
  - synthetic/two: name -> pass (gold name; None)
  - synthetic/two: name -> pass (gold keep; None)
  - synthetic/two: name -> pass (gold name; None)
  - synthetic/two: name -> pass (gold keep; None)
  - synthetic/two: name -> pass (gold keep; None)
- **v2**: 10 changed, 0 now match gold
  - household/two: name -> pass (gold keep; None)
  - household/two: name -> pass (gold keep; None)
  - household/two: name -> pass (gold keep; None)
  - household/two: name -> pass (gold keep; None)
  - synthetic/two: name -> pass (gold name; None)
  - synthetic/two: name -> pass (gold name; None)
  - synthetic/two: name -> pass (gold keep; None)
  - synthetic/two: name -> pass (gold name; None)
  - synthetic/two: name -> pass (gold keep; None)
  - synthetic/two: name -> pass (gold keep; None)
- **v2+recency**: 11 changed, 0 now match gold
  - household/two: name -> pass (gold keep; None)
  - household/two: name -> pass (gold keep; None)
  - synthetic/two: name -> pass (gold name; None)
  - synthetic/two: pass -> name (gold keep; 6 recency)
  - synthetic/two: pass -> name (gold keep; 6 recency)
  - synthetic/two: pass -> name (gold keep; 6 recency)
  - synthetic/two: pass -> name (gold keep; 6 recency)
  - synthetic/two: name -> pass (gold name; None)
  - synthetic/two: name -> pass (gold name; None)
  - synthetic/two: name -> pass (gold keep; None)
  - synthetic/two: pass -> name (gold keep; 6 recency)
