# TPE comparison results

> Absolute timings are machine-specific. Compare only runs captured on the same machine. Timing and quality are reported independently; no combined score is calculated.

Benchmark protocol(s): checkpoint, curated, quick.

## categorical / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 27677.1 | 28357.3 | 29618.3 | 36131.0 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 27524.1 | 27961.3 | 29802.7 | 36331.8 | 0/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 27333.5 | 27605.7 | 28211.7 | 36585.2 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 26412.8 | 26683.7 | 28466.5 | 37860.4 | 4/4 |

## categorical / quality

History: 1000; dimensions: 1; budget: 25; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.100000 | 81.2% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.100000 | 81.2% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.100000 | 81.2% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.100000 | 81.2% | 0.000000 |

## categorical / quality

History: 1000; dimensions: 1; budget: 50; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## categorical / quality

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## categorical / quality

History: 1000; dimensions: 1; budget: 250; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## categorical / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 209820 | 3/4 | 484.2 | 488.1 | 495.0 | 2065148.0 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 262144 | 3/4 | 472.1 | 475.3 | 479.2 | 2118036.6 | 2/4 |
| parzen/full (scalar-f64) | supported | 262144 | 3/4 | 484.7 | 487.8 | 501.2 | 2063023.2 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 210045 | 3/4 | 471.7 | 475.2 | 497.4 | 2120029.4 | 2/4 |

## conditional / cycle

History: 1000; dimensions: 2; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 177675.3 | 179912.4 | 181994.0 | 5628.2 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 180546.6 | 182179.8 | 184073.7 | 5538.7 | 0/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 25560.8 | 25990.6 | 26649.2 | 39122.4 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 23957.8 | 24188.0 | 25433.8 | 41740.1 | 4/4 |

## conditional / quality

History: 1000; dimensions: 2; budget: 25; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.250000 | 0.031622 | 0.250000 | 12.5% | 0.250000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.031622 | 0.250000 | 12.5% | 0.250000 |
| parzen/full (scalar-f64) | supported | 32 | 0.250000 | 0.031622 | 0.250000 | 12.5% | 0.250000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.031622 | 0.250000 | 12.5% | 0.250000 |

## conditional / quality

History: 1000; dimensions: 2; budget: 50; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.250000 | 0.000683 | 0.250000 | 18.8% | 0.250000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.000683 | 0.250000 | 18.8% | 0.250000 |
| parzen/full (scalar-f64) | supported | 32 | 0.250000 | 0.000683 | 0.250000 | 18.8% | 0.250000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.000683 | 0.250000 | 18.8% | 0.250000 |

## conditional / quality

History: 1000; dimensions: 2; budget: 100; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.250000 | 0.000130 | 0.250000 | 31.2% | 0.250000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.000130 | 0.250000 | 31.2% | 0.250000 |
| parzen/full (scalar-f64) | supported | 32 | 0.250000 | 0.000130 | 0.250000 | 31.2% | 0.250000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.000130 | 0.250000 | 31.2% | 0.250000 |

## conditional / quality

History: 1000; dimensions: 2; budget: 250; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.031199 | 0.000000 | 0.250000 | 46.9% | 0.031199 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.031199 | 0.000000 | 0.250000 | 46.9% | 0.031199 |
| parzen/full (scalar-f64) | supported | 32 | 0.031199 | 0.000000 | 0.250000 | 46.9% | 0.031199 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.031199 | 0.000000 | 0.250000 | 46.9% | 0.031199 |

## conditional / suggest

History: 1000; dimensions: 2; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 1629 | 3/4 | 87297.4 | 87730.1 | 88184.3 | 11455.1 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 1596 | 3/4 | 86976.7 | 88183.4 | 90412.7 | 11497.3 | 0/4 |
| parzen/full (scalar-f64) | supported | 262144 | 3/4 | 389.5 | 391.6 | 397.1 | 2567418.7 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 262144 | 3/4 | 373.3 | 375.9 | 450.3 | 2678877.6 | 4/4 |

## correlated-numeric / cycle

History: 1000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 402005.8 | 404773.6 | 408977.9 | 2487.5 | 2/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 400410.7 | 405161.1 | 408649.9 | 2497.4 | 2/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 745871.9 | 752941.1 | 756199.6 | 1340.7 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 749779.8 | 756727.0 | 758756.0 | 1333.7 | 0/4 |

## correlated-numeric / cycle

History: 1000; dimensions: 8; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 50 | 3/4 | 944714.5 | 949092.0 | 957814.0 | 1058.5 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 637323.7 | 642194.0 | 646160.7 | 1569.1 | 4/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 1170989.3 | 1179455.9 | 1189071.7 | 854.0 | 0/4 |

## correlated-numeric / quality

History: 1000; dimensions: 4; budget: 25; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 146.099762 | 42.585723 | 366.496064 | 0.0% | 146.099762 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 146.099762 | 42.585723 | 366.496064 | 0.0% | 146.099762 |
| parzen/full (scalar-f64) | supported | 32 | 146.099762 | 42.585723 | 366.496064 | 0.0% | 146.099762 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 146.099762 | 42.585723 | 366.496064 | 0.0% | 146.099762 |

## correlated-numeric / quality

History: 1000; dimensions: 4; budget: 50; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 38.665840 | 9.470001 | 100.480363 | 0.0% | 38.665840 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 38.665840 | 9.470001 | 100.480363 | 0.0% | 38.665840 |
| parzen/full (scalar-f64) | supported | 32 | 38.665840 | 9.470001 | 100.480363 | 0.0% | 38.665840 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 38.665840 | 9.470001 | 100.480363 | 0.0% | 38.665840 |

## correlated-numeric / quality

History: 1000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 7.513469 | 3.716869 | 26.282546 | 0.0% | 7.513469 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 7.513469 | 3.716869 | 26.282546 | 0.0% | 7.513469 |
| parzen/full (scalar-f64) | supported | 32 | 7.513469 | 3.716869 | 26.282546 | 0.0% | 7.513469 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 7.513469 | 3.716869 | 26.282546 | 0.0% | 7.513469 |

## correlated-numeric / quality

History: 1000; dimensions: 4; budget: 250; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 2.989532 | 1.953479 | 5.466015 | 0.0% | 2.989532 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 2.989532 | 1.953479 | 5.466015 | 0.0% | 2.989532 |
| parzen/full (scalar-f64) | supported | 32 | 2.989532 | 1.953479 | 5.466015 | 0.0% | 2.989532 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 2.989532 | 1.953479 | 5.466015 | 0.0% | 2.989532 |

## correlated-numeric / suggest

History: 1000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 738 | 3/4 | 179434.1 | 181438.2 | 182265.7 | 5573.1 | 4/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 738 | 3/4 | 181831.6 | 182933.9 | 183817.1 | 5499.6 | 0/4 |
| parzen/full (scalar-f64) | supported | 369 | 3/4 | 352106.0 | 355561.6 | 357440.4 | 2840.1 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 366 | 3/4 | 359067.6 | 360790.5 | 362831.4 | 2785.0 | 0/4 |

## correlated-numeric / suggest

History: 1000; dimensions: 8; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 214 | 3/4 | 917563.7 | 921302.9 | 935344.0 | 1089.8 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 620 | 3/4 | 199225.3 | 200513.1 | 212375.7 | 5019.4 | 4/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 316 | 3/4 | 392117.9 | 395386.7 | 403371.2 | 2550.3 | 0/4 |

## independent-float / cycle

History: 10; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 50 | 3/4 | 39757.0 | 40598.4 | 41877.2 | 25152.8 | 0/4 |
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 44951.1 | 45621.5 | 47299.5 | 22246.4 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 50 | 3/4 | 37854.9 | 38461.1 | 40124.0 | 26416.6 | 4/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 42979.2 | 44037.9 | 45161.1 | 23267.0 | 0/4 |

## independent-float / cycle

History: 100; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 50 | 3/4 | 151476.3 | 153784.1 | 156652.6 | 6601.7 | 0/4 |
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 171663.8 | 173891.6 | 175804.3 | 5825.3 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 50 | 3/4 | 137922.7 | 139777.2 | 142959.5 | 7250.4 | 4/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 159765.1 | 161158.3 | 163510.8 | 6259.2 | 0/4 |

## independent-float / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 154016.1 | 155978.4 | 157815.4 | 6492.8 | 4/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 156611.0 | 157396.4 | 159677.5 | 6385.2 | 0/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 280625.6 | 285631.5 | 288226.2 | 3563.5 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 283069.3 | 285467.3 | 288568.4 | 3532.7 | 0/4 |

## independent-float / cycle

History: 1000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| hyperopt | supported | 50 | 3/4 | 1704403.4 | 1718259.9 | 1731038.0 | 586.7 | 0/4 |
| optimizer | supported | 50 | 3/4 | 703672.1 | 711551.9 | 715835.6 | 1421.1 | 4/4 |
| parzen/bounded (pulp-avx2-fma) | supported | 50 | 6/8 | 729191.7 | 739809.4 | 745361.1 | 1371.4 | 0/4 |
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 793869.7 | 799908.4 | 803618.5 | 1259.7 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 50 | 6/8 | 1361282.8 | 1393214.2 | 1403079.7 | 734.6 | 0/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 1517749.0 | 1525471.4 | 1537644.3 | 658.9 | 0/4 |
| tpe | supported | 50 | 3/4 | 865690.8 | 877231.7 | 887537.8 | 1155.1 | 0/4 |

## independent-float / cycle

History: 1000; dimensions: 8; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 50 | 3/4 | 1495990.7 | 1513024.1 | 1529590.7 | 668.5 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 1611023.5 | 1622628.0 | 1631269.4 | 620.7 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 50 | 3/4 | 2851044.0 | 2873190.0 | 3999952.7 | 350.7 | 0/4 |
| parzen/full (scalar-f64) | supported | 32 | 3/4 | 3125492.8 | 3142077.2 | 3153976.2 | 319.9 | 0/4 |

## independent-float / cycle

History: 1000; dimensions: 16; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 50 | 3/4 | 3053437.1 | 3136460.4 | 3171390.2 | 327.5 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 3313465.4 | 3328082.2 | 3340532.1 | 301.8 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 17 | 3/4 | 5856818.7 | 5893028.4 | 5938385.9 | 170.7 | 0/4 |
| parzen/full (scalar-f64) | supported | 16 | 3/4 | 6326363.6 | 6367513.3 | 6410238.1 | 158.1 | 0/4 |

## independent-float / cycle

History: 10000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 70 | 7/8 | 6941368.2 | 7025503.5 | 7077003.5 | 144.1 | 0/8 |
| parzen/bounded (pulp-avx2-fma) | supported | 100 | 7/8 | 705863.6 | 725326.8 | 734255.6 | 1416.7 | 8/8 |
| parzen/bounded (scalar-f64) | supported | 100 | 7/8 | 774447.5 | 782644.9 | 789247.2 | 1291.2 | 0/8 |
| parzen/full (pulp-avx2-fma) | supported | 17 | 7/8 | 14754985.7 | 14877536.4 | 14958213.0 | 67.8 | 0/8 |
| parzen/full (scalar-f64) | supported | 30 | 7/8 | 16287861.5 | 16382199.4 | 16447859.7 | 61.4 | 0/8 |

## independent-float / cycle

History: 100000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 50 | 3/4 | 717225.8 | 724275.8 | 726858.5 | 1394.3 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 772259.1 | 780648.9 | 790912.0 | 1294.9 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 1 | 3/4 | 160916098.0 | 162158398.0 | 164142406.0 | 6.2 | 0/4 |
| parzen/full (scalar-f64) | supported | 1 | 3/4 | 175963831.0 | 177201591.0 | 178996755.0 | 5.7 | 0/4 |

## independent-float / memory

History: 1000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Retained after ingest | Retained at cycle end | Cycle bytes/op | Peak live bytes | Peak RSS bytes |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 1247168 | 1616144 | 147658.7 | 1633762 | 6160384 |
| parzen/bounded (pulp-avx2-fma) | supported | 205264 | 463464 | 1546.6 | 463530 | 4587520 |
| parzen/full (pulp-avx2-fma) | supported | 197928 | 856600 | 121411.4 | 873050 | 4587520 |

## independent-float / profile

Profile workload: `cycle`.

History: 10000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Workload | Operations | Start observations | End observations | Profile seconds |
|---|---|---|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | cycle | 57227 | 10000 | 67227 | 30.000 |
| parzen/bounded (scalar-f64) | supported | cycle | 50053 | 10000 | 60053 | 30.000 |
| parzen/full (pulp-avx2-fma) | supported | cycle | 2006 | 10000 | 12006 | 30.008 |
| parzen/full (scalar-f64) | supported | cycle | 1830 | 10000 | 11830 | 30.001 |

## independent-float / profile

Profile workload: `fixed-suggest`.

History: 10000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Workload | Operations | Start observations | End observations | Profile seconds |
|---|---|---|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | fixed-suggest | 61826 | 10000 | 10000 | 30.000 |
| parzen/bounded (scalar-f64) | supported | fixed-suggest | 55968 | 10000 | 10000 | 30.000 |
| parzen/full (pulp-avx2-fma) | supported | fixed-suggest | 2690 | 10000 | 10000 | 30.006 |
| parzen/full (scalar-f64) | supported | fixed-suggest | 2363 | 10000 | 10000 | 30.008 |

## independent-float / quality

History: 1000; dimensions: 4; budget: 25; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 32 | 17.319447 | 6.845352 | 33.206629 | 0.0% | 17.319447 |
| parzen/bounded (scalar-f64) | supported | 32 | 17.319447 | 6.845352 | 33.206629 | 0.0% | 17.319447 |
| parzen/full (pulp-avx2-fma) | supported | 32 | 17.319447 | 6.845352 | 33.206629 | 0.0% | 17.319447 |
| parzen/full (scalar-f64) | supported | 32 | 17.319447 | 6.845352 | 33.206629 | 0.0% | 17.319447 |

## independent-float / quality

History: 1000; dimensions: 4; budget: 50; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 32 | 8.326971 | 3.448432 | 12.132915 | 0.0% | 8.326971 |
| parzen/bounded (scalar-f64) | supported | 32 | 8.326971 | 3.448432 | 12.132915 | 0.0% | 8.326971 |
| parzen/full (pulp-avx2-fma) | supported | 32 | 8.326971 | 3.448432 | 12.132915 | 0.0% | 8.326971 |
| parzen/full (scalar-f64) | supported | 32 | 8.326971 | 3.448432 | 12.132915 | 0.0% | 8.326971 |

## independent-float / quality

History: 1000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 32 | 1.320619 | 0.592668 | 3.134938 | 0.0% | 1.320619 |
| parzen/bounded (scalar-f64) | supported | 32 | 1.320619 | 0.592668 | 3.134938 | 0.0% | 1.320619 |
| parzen/full (pulp-avx2-fma) | supported | 32 | 1.320619 | 0.592668 | 3.134938 | 0.0% | 1.320619 |
| parzen/full (scalar-f64) | supported | 32 | 1.320619 | 0.592668 | 3.134938 | 0.0% | 1.320619 |

## independent-float / quality

History: 1000; dimensions: 4; budget: 250; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 32 | 0.135747 | 0.038941 | 0.404591 | 0.0% | 0.135747 |
| parzen/bounded (scalar-f64) | supported | 32 | 0.135747 | 0.038941 | 0.404591 | 0.0% | 0.135747 |
| parzen/full (pulp-avx2-fma) | supported | 32 | 0.135747 | 0.038941 | 0.404591 | 0.0% | 0.135747 |
| parzen/full (scalar-f64) | supported | 32 | 0.135747 | 0.038941 | 0.404591 | 0.0% | 0.135747 |

## independent-float / suggest

History: 10; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 9264 | 3/4 | 14375.6 | 14457.6 | 14767.0 | 69562.2 | 1/4 |
| parzen/bounded (scalar-f64) | supported | 8544 | 3/4 | 15291.8 | 15390.7 | 15508.3 | 65394.5 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 8846 | 3/4 | 14386.6 | 14592.2 | 14706.8 | 69509.2 | 3/4 |
| parzen/full (scalar-f64) | supported | 8108 | 3/4 | 15307.5 | 15377.5 | 15613.7 | 65327.5 | 0/4 |

## independent-float / suggest

History: 100; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 1500 | 3/4 | 74045.8 | 74611.8 | 76055.4 | 13505.2 | 3/4 |
| parzen/bounded (scalar-f64) | supported | 1416 | 3/4 | 82994.8 | 83635.7 | 84270.2 | 12048.9 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 1660 | 3/4 | 73940.9 | 74930.0 | 76576.6 | 13524.3 | 1/4 |
| parzen/full (scalar-f64) | supported | 1530 | 3/4 | 83359.6 | 83771.0 | 85936.6 | 11996.2 | 0/4 |

## independent-float / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 1244 | 3/4 | 93404.6 | 93894.9 | 94958.5 | 10706.1 | 2/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 1280 | 3/4 | 92569.0 | 94744.1 | 97432.6 | 10802.8 | 2/4 |
| parzen/full (scalar-f64) | supported | 722 | 3/4 | 183242.9 | 185475.6 | 188000.5 | 5457.2 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 748 | 3/4 | 180076.4 | 186967.5 | 190588.8 | 5553.2 | 0/4 |

## independent-float / suggest

History: 1000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| hyperopt | supported | 118 | 3/4 | 1664368.8 | 1688344.9 | 1703666.8 | 600.8 | 0/4 |
| optimizer | supported | 284 | 3/4 | 692418.2 | 698301.9 | 703347.1 | 1444.2 | 0/4 |
| parzen/bounded (pulp-avx2-fma) | supported | 256, 258 | 6/8 | 467340.8 | 486437.6 | 491593.8 | 2139.8 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 240 | 3/4 | 545200.5 | 552405.7 | 555922.6 | 1834.2 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 144 | 6/8 | 954786.9 | 1002035.3 | 1013107.7 | 1047.4 | 0/4 |
| parzen/full (scalar-f64) | supported | 130 | 3/4 | 1121184.1 | 1131517.7 | 1138452.8 | 891.9 | 0/4 |
| tpe | supported | 222 | 3/4 | 802213.2 | 809931.7 | 818551.8 | 1246.6 | 0/4 |

## independent-float / suggest

History: 1000; dimensions: 8; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 122 | 3/4 | 1043807.5 | 1049758.6 | 1057733.0 | 958.0 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 116 | 3/4 | 1168633.5 | 1174277.9 | 1184778.7 | 855.7 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 70 | 3/4 | 2116487.1 | 2130111.3 | 2146292.0 | 472.5 | 0/4 |
| parzen/full (scalar-f64) | supported | 64 | 3/4 | 2327764.2 | 2361670.1 | 2373884.3 | 429.6 | 0/4 |

## independent-float / suggest

History: 1000; dimensions: 16; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 60 | 3/4 | 2166794.5 | 2187698.8 | 2195763.8 | 461.5 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 56 | 3/4 | 2376191.8 | 2404757.7 | 2434280.2 | 420.8 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 34 | 3/4 | 4355854.9 | 4389092.7 | 4438492.5 | 229.6 | 0/4 |
| parzen/full (scalar-f64) | supported | 32 | 3/4 | 4755899.5 | 4809062.4 | 4828641.0 | 210.3 | 0/4 |

## independent-float / suggest

History: 10000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 36 | 7/8 | 6982913.5 | 7034039.5 | 7098063.2 | 143.2 | 0/8 |
| parzen/bounded (pulp-avx2-fma) | supported | 654 | 7/8 | 455634.9 | 460751.9 | 464742.4 | 2194.7 | 8/8 |
| parzen/bounded (scalar-f64) | supported | 606 | 7/8 | 520835.6 | 528356.7 | 530786.5 | 1920.0 | 0/8 |
| parzen/full (pulp-avx2-fma) | supported | 17 | 7/8 | 10911348.3 | 11002589.9 | 16321666.6 | 91.6 | 0/8 |
| parzen/full (scalar-f64) | supported | 30 | 7/8 | 12301494.0 | 12429481.2 | 12496525.4 | 81.3 | 0/8 |

## independent-float / suggest

History: 100000; dimensions: 4; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 256 | 3/4 | 439830.1 | 444222.4 | 450715.7 | 2273.6 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 238 | 3/4 | 496614.9 | 507422.0 | 509292.1 | 2013.6 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 1 | 3/4 | 161074772.0 | 161851520.0 | 164618337.0 | 6.2 | 0/4 |
| parzen/full (scalar-f64) | supported | 1 | 3/4 | 175434689.0 | 176786371.0 | 178123264.0 | 5.7 | 0/4 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 8.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 25 | 1/2 | 139534.9 | 144000.6 | 145954.7 | 7166.7 | 1/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 141618.6 | 142633.4 | 145309.1 | 7061.2 | 1/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 144445.8 | 145566.3 | 147280.3 | 6923.0 | 0/2 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 16.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 25 | 1/2 | 143210.6 | 146447.1 | 147523.5 | 6982.7 | 2/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 167628.0 | 168705.2 | 171862.9 | 5965.6 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 188421.3 | 190673.0 | 192628.6 | 5307.3 | 0/2 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 32.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 25 | 1/2 | 139737.4 | 144730.7 | 152996.9 | 7156.3 | 2/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 186978.9 | 189005.8 | 190556.6 | 5348.2 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 246605.9 | 248435.6 | 256375.3 | 4055.1 | 0/2 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 64.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 25 | 1/2 | 141012.2 | 143723.1 | 147345.2 | 7091.6 | 2/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 254772.1 | 255613.8 | 260916.7 | 3925.1 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 406398.2 | 409496.2 | 413400.0 | 2460.6 | 0/2 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 50 | 3/4 | 142310.9 | 145009.1 | 149563.4 | 7026.9 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 519548.4 | 523657.9 | 526453.2 | 1924.7 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 50 | 6/8 | 520083.4 | 525410.6 | 532031.1 | 1922.8 | 0/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 711338.7 | 719187.3 | 724117.8 | 1405.8 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 50 | 6/8 | 710190.9 | 717991.3 | 723274.0 | 1408.1 | 0/4 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 256.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 25 | 1/2 | 142279.8 | 146594.8 | 147609.2 | 7028.4 | 2/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 554989.7 | 558645.0 | 562423.9 | 1801.8 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 899464.0 | 900754.8 | 904660.1 | 1111.8 | 0/2 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 1024.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 25 | 1/2 | 140694.2 | 144359.5 | 145446.3 | 7107.6 | 2/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 657759.0 | 661889.5 | 663770.7 | 1520.3 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 1219604.5 | 1221611.7 | 1234498.0 | 819.9 | 0/2 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 4096.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 25 | 1/2 | 140349.8 | 144180.6 | 144933.9 | 7125.1 | 2/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 684668.4 | 688121.7 | 693502.2 | 1460.6 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 1292951.4 | 1303908.5 | 1309216.6 | 773.4 | 0/2 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 100001.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 25 | 1/2 | 143491.4 | 148226.8 | 156614.6 | 6969.1 | 2/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 705725.7 | 707897.4 | 712803.5 | 1417.0 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 25 | 1/2 | 1370651.7 | 1375635.4 | 1384002.4 | 729.6 | 0/2 |

## integer / memory

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Retained after ingest | Retained at cycle end | Cycle bytes/op | Peak live bytes | Peak RSS bytes |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 780922 | 1102784 | 41108.7 | 1120306 | 5111808 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 85608 | 175448 | 753.3 | 175464 | 3932160 |
| parzen/full (scalar-f64-policy-fallback) | supported | 114832 | 319840 | 30891.4 | 336240 | 4194304 |

## integer / quality

History: 1000; dimensions: 1; budget: 25; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 1.000000 | 0.000000 | 9.000000 | 21.9% | 1.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 1.000000 | 0.000000 | 9.000000 | 21.9% | 1.000000 |
| parzen/full (scalar-f64) | supported | 32 | 1.000000 | 0.000000 | 9.000000 | 21.9% | 1.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 1.000000 | 0.000000 | 9.000000 | 21.9% | 1.000000 |

## integer / quality

History: 1000; dimensions: 1; budget: 50; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 1.000000 | 62.5% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 1.000000 | 62.5% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 1.000000 | 62.5% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 1.000000 | 62.5% | 0.000000 |

## integer / quality

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## integer / quality

History: 1000; dimensions: 1; budget: 250; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 8.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 348 | 1/2 | 135168.4 | 135683.9 | 136781.4 | 7398.2 | 0/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 23100 | 1/2 | 1152.3 | 1164.9 | 1173.8 | 867802.2 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 25080 | 1/2 | 1055.8 | 1060.1 | 1064.6 | 947150.8 | 2/2 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 16.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 334 | 1/2 | 141158.1 | 143252.8 | 143790.2 | 7084.3 | 0/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 21460 | 1/2 | 1312.8 | 1323.6 | 1372.9 | 761712.3 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 41400 | 1/2 | 1154.7 | 1170.4 | 1176.2 | 866023.0 | 2/2 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 32.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 332 | 1/2 | 137088.5 | 138228.6 | 138498.7 | 7294.6 | 0/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 15240 | 1/2 | 1569.4 | 1575.7 | 1692.4 | 637183.3 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 23086 | 1/2 | 1317.0 | 1341.4 | 1346.9 | 759285.3 | 2/2 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 64.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 338 | 1/2 | 140546.0 | 141062.2 | 141750.9 | 7115.1 | 0/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 19740 | 1/2 | 1686.9 | 1692.6 | 1705.3 | 592800.3 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 20160 | 1/2 | 1502.9 | 1510.2 | 1519.8 | 665401.7 | 2/2 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 1370 | 3/4 | 141682.4 | 142948.1 | 143636.3 | 7058.0 | 0/4 |
| parzen/bounded (scalar-f64) | supported | 63504 | 3/4 | 1743.8 | 1761.9 | 1789.2 | 573467.5 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 70560, 71280 | 6/8 | 1619.4 | 1634.9 | 1652.5 | 617506.1 | 0/4 |
| parzen/full (scalar-f64) | supported | 117936 | 3/4 | 1570.0 | 1581.0 | 1591.9 | 636953.7 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 123396 | 6/8 | 1512.6 | 1524.8 | 1537.7 | 661122.8 | 4/4 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 256.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 340 | 1/2 | 139781.1 | 143015.3 | 143935.9 | 7154.0 | 0/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 16512 | 1/2 | 2022.4 | 2043.8 | 2076.5 | 494453.1 | 2/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 13200 | 1/2 | 2444.5 | 2465.8 | 2503.6 | 409085.9 | 0/2 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 1024.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 338 | 1/2 | 139127.4 | 140650.9 | 141705.1 | 7187.7 | 0/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 936 | 1/2 | 27917.1 | 28045.6 | 28253.8 | 35820.3 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 1008 | 1/2 | 26200.8 | 26389.7 | 26540.4 | 38166.8 | 2/2 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 4096.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 322 | 1/2 | 140173.8 | 141223.0 | 141772.1 | 7134.0 | 0/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 240 | 1/2 | 104770.7 | 105730.8 | 105810.6 | 9544.7 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 304 | 1/2 | 95177.5 | 95690.2 | 96132.8 | 10506.7 | 2/2 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 100001.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| optimizer | supported | 336 | 1/2 | 141981.5 | 142590.5 | 143312.2 | 7043.2 | 2/2 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 72 | 1/2 | 598658.4 | 599980.2 | 601815.0 | 1670.4 | 0/2 |
| parzen/full (scalar-f64-policy-fallback) | supported | 36 | 1/2 | 1248869.1 | 1254458.2 | 1264451.5 | 800.7 | 0/2 |

## linear-float / quality

History: 1000; dimensions: 1; budget: 25; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.007072 | 0.000246 | 0.065781 | 62.5% | 0.007072 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.007072 | 0.000246 | 0.065781 | 62.5% | 0.007072 |
| parzen/full (scalar-f64) | supported | 32 | 0.007072 | 0.000246 | 0.065781 | 62.5% | 0.007072 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.007072 | 0.000246 | 0.065781 | 62.5% | 0.007072 |

## linear-float / quality

History: 1000; dimensions: 1; budget: 50; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000806 | 0.000020 | 0.004241 | 96.9% | 0.000806 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000806 | 0.000020 | 0.004241 | 96.9% | 0.000806 |
| parzen/full (scalar-f64) | supported | 32 | 0.000806 | 0.000020 | 0.004241 | 96.9% | 0.000806 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000806 | 0.000020 | 0.004241 | 96.9% | 0.000806 |

## linear-float / quality

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000043 | 0.000001 | 0.000431 | 100.0% | 0.000043 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000043 | 0.000001 | 0.000431 | 100.0% | 0.000043 |
| parzen/full (scalar-f64) | supported | 32 | 0.000043 | 0.000001 | 0.000431 | 100.0% | 0.000043 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000043 | 0.000001 | 0.000431 | 100.0% | 0.000043 |

## linear-float / quality

History: 1000; dimensions: 1; budget: 250; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000002 | 0.000000 | 0.000015 | 100.0% | 0.000002 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000002 | 0.000000 | 0.000015 | 100.0% | 0.000002 |
| parzen/full (scalar-f64) | supported | 32 | 0.000002 | 0.000000 | 0.000015 | 100.0% | 0.000002 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000002 | 0.000000 | 0.000015 | 100.0% | 0.000002 |

## log-float / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 158655.5 | 160730.3 | 164152.8 | 6303.0 | 3/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 160421.1 | 162056.1 | 163531.4 | 6233.6 | 1/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 290172.0 | 294008.7 | 297074.0 | 3446.2 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 289501.0 | 293505.5 | 295519.2 | 3454.2 | 0/4 |

## log-float / quality

History: 1000; dimensions: 1; budget: 25; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.004469 | 0.000187 | 0.046946 | 56.2% | 0.004469 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.004469 | 0.000187 | 0.046946 | 56.2% | 0.004469 |
| parzen/full (scalar-f64) | supported | 32 | 0.004469 | 0.000187 | 0.046946 | 56.2% | 0.004469 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.004469 | 0.000187 | 0.046946 | 56.2% | 0.004469 |

## log-float / quality

History: 1000; dimensions: 1; budget: 50; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000540 | 0.000042 | 0.002479 | 100.0% | 0.000540 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000540 | 0.000042 | 0.002479 | 100.0% | 0.000540 |
| parzen/full (scalar-f64) | supported | 32 | 0.000540 | 0.000042 | 0.002479 | 100.0% | 0.000540 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000540 | 0.000042 | 0.002479 | 100.0% | 0.000540 |

## log-float / quality

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000047 | 0.000002 | 0.000185 | 100.0% | 0.000047 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000047 | 0.000002 | 0.000185 | 100.0% | 0.000047 |
| parzen/full (scalar-f64) | supported | 32 | 0.000047 | 0.000002 | 0.000185 | 100.0% | 0.000047 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000047 | 0.000002 | 0.000185 | 100.0% | 0.000047 |

## log-float / quality

History: 1000; dimensions: 1; budget: 250; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000001 | 0.000000 | 0.000010 | 100.0% | 0.000001 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000001 | 0.000000 | 0.000010 | 100.0% | 0.000001 |
| parzen/full (scalar-f64) | supported | 32 | 0.000001 | 0.000000 | 0.000010 | 100.0% | 0.000001 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000001 | 0.000000 | 0.000010 | 100.0% | 0.000001 |

## log-float / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 1190 | 3/4 | 97389.8 | 97802.7 | 98258.2 | 10268.0 | 3/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 1284 | 3/4 | 96558.9 | 100180.3 | 101696.6 | 10356.4 | 1/4 |
| parzen/full (scalar-f64) | supported | 724 | 3/4 | 191918.1 | 193074.5 | 194177.8 | 5210.6 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 708 | 3/4 | 189858.5 | 193419.5 | 197760.3 | 5267.1 | 0/4 |

## mixed-independent / cycle

History: 1000; dimensions: 3; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 781789.8 | 791066.4 | 797746.6 | 1279.1 | 2/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 785297.7 | 792016.6 | 797698.8 | 1273.4 | 2/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 1567152.6 | 1576837.5 | 1599595.2 | 638.1 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 1565048.4 | 1576928.5 | 1586496.5 | 639.0 | 0/4 |

## mixed-independent / quality

History: 1000; dimensions: 3; budget: 25; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 1.907527 | 1.071005 | 4.831794 | 0.0% | 1.907527 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 1.907527 | 1.071005 | 4.831794 | 0.0% | 1.907527 |
| parzen/full (scalar-f64) | supported | 32 | 1.907527 | 1.071005 | 4.831794 | 0.0% | 1.907527 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 1.907527 | 1.071005 | 4.831794 | 0.0% | 1.907527 |

## mixed-independent / quality

History: 1000; dimensions: 3; budget: 50; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 1.186190 | 0.462368 | 2.722823 | 0.0% | 1.186190 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 1.186190 | 0.462368 | 2.722823 | 0.0% | 1.186190 |
| parzen/full (scalar-f64) | supported | 32 | 1.186190 | 0.462368 | 2.722823 | 0.0% | 1.186190 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 1.186190 | 0.462368 | 2.722823 | 0.0% | 1.186190 |

## mixed-independent / quality

History: 1000; dimensions: 3; budget: 100; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 1.007674 | 0.078161 | 1.096562 | 0.0% | 1.007674 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 1.007674 | 0.078161 | 1.096562 | 0.0% | 1.007674 |
| parzen/full (scalar-f64) | supported | 32 | 1.007674 | 0.078161 | 1.096562 | 0.0% | 1.007674 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 1.007674 | 0.078161 | 1.096562 | 0.0% | 1.007674 |

## mixed-independent / quality

History: 1000; dimensions: 3; budget: 250; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.368568 | 0.000826 | 1.004520 | 28.1% | 0.368568 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.368568 | 0.000826 | 1.004520 | 28.1% | 0.368568 |
| parzen/full (scalar-f64) | supported | 32 | 0.368568 | 0.000826 | 1.004520 | 28.1% | 0.368568 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.368568 | 0.000826 | 1.004520 | 28.1% | 0.368568 |

## mixed-independent / suggest

History: 1000; dimensions: 3; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 1428 | 3/4 | 110860.4 | 112467.3 | 113613.3 | 9020.3 | 3/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 1428 | 3/4 | 112134.6 | 112887.8 | 113625.6 | 8917.9 | 1/4 |
| parzen/full (scalar-f64) | supported | 732 | 3/4 | 234733.1 | 239070.6 | 240929.0 | 4260.2 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 708 | 3/4 | 235368.6 | 238925.3 | 243240.9 | 4248.7 | 0/4 |

## stepped-float / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 681591.5 | 684830.6 | 689349.7 | 1467.2 | 3/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 680707.9 | 689444.8 | 710254.2 | 1469.1 | 1/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 1270772.7 | 1278840.5 | 1286800.6 | 786.9 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 1258134.0 | 1273110.1 | 1286074.2 | 794.8 | 0/4 |

## stepped-float / quality

History: 1000; dimensions: 1; budget: 25; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.250000 | 84.4% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.250000 | 84.4% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.250000 | 84.4% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.250000 | 84.4% | 0.000000 |

## stepped-float / quality

History: 1000; dimensions: 1; budget: 50; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## stepped-float / quality

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## stepped-float / quality

History: 1000; dimensions: 1; budget: 250; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## stepped-float / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 302 | 3/4 | 596640.1 | 603393.1 | 607038.1 | 1676.1 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 284 | 3/4 | 594430.2 | 598655.5 | 603691.2 | 1682.3 | 4/4 |
| parzen/full (scalar-f64) | supported | 148 | 3/4 | 1149599.5 | 1154432.5 | 1223920.5 | 869.9 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 160 | 3/4 | 1139387.5 | 1148664.4 | 1157472.9 | 877.7 | 0/4 |

## stepped-integer / cycle

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 50 | 3/4 | 304397.4 | 313322.2 | 319236.4 | 3285.2 | 1/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 300022.7 | 304913.4 | 312439.6 | 3333.1 | 3/4 |
| parzen/full (scalar-f64) | supported | 50 | 3/4 | 347884.2 | 352419.8 | 356490.5 | 2874.5 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 50 | 3/4 | 335202.1 | 337545.0 | 343798.5 | 2983.3 | 0/4 |

## stepped-integer / quality

History: 1000; dimensions: 1; budget: 25; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 25.000000 | 84.4% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 25.000000 | 84.4% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 25.000000 | 84.4% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 25.000000 | 84.4% | 0.000000 |

## stepped-integer / quality

History: 1000; dimensions: 1; budget: 50; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## stepped-integer / quality

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## stepped-integer / quality

History: 1000; dimensions: 1; budget: 250; integer cardinality: 201.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## stepped-integer / suggest

History: 1000; dimensions: 1; budget: 100; integer cardinality: 201.

| Backend | Status | Iterations/sample | Reused calibrations | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 135904 | 3/4 | 1422.0 | 1497.6 | 1691.7 | 703221.0 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 137392 | 3/4 | 1401.8 | 1413.8 | 1419.7 | 713383.1 | 0/4 |
| parzen/full (scalar-f64) | supported | 134652 | 3/4 | 1338.2 | 1355.2 | 1424.5 | 747273.9 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 138460 | 3/4 | 1322.5 | 1330.5 | 1339.5 | 756150.0 | 4/4 |
