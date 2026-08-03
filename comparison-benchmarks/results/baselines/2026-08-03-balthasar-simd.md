# TPE comparison results

> Absolute timings are machine-specific. Compare only runs captured on the same machine. Timing and quality are reported independently; no combined score is calculated.

Benchmark protocol(s): checkpoint, curated, quick.

## categorical / cycle

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 27424.8 | 27823.2 | 28435.0 | 36463.3 | 0/8 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 27403.4 | 27820.2 | 28542.4 | 36491.8 | 0/8 |
| parzen/full (scalar-f64) | supported | 27038.1 | 27440.0 | 28139.8 | 36984.8 | 0/8 |
| parzen/full (scalar-f64-policy-fallback) | supported | 26210.2 | 26656.9 | 28274.4 | 38153.1 | 8/8 |

## categorical / quality

History: 1000; dimensions: 1; budget: 25.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.100000 | 81.2% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.100000 | 81.2% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.100000 | 81.2% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.100000 | 81.2% | 0.000000 |

## categorical / quality

History: 1000; dimensions: 1; budget: 50.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## categorical / quality

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## categorical / quality

History: 1000; dimensions: 1; budget: 250.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## categorical / suggest

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 484.4 | 491.4 | 499.9 | 2064285.7 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 469.3 | 472.7 | 478.2 | 2130874.7 | 3/4 |
| parzen/full (scalar-f64) | supported | 481.3 | 486.8 | 507.4 | 2077893.5 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 472.0 | 474.9 | 521.9 | 2118641.0 | 1/4 |

## conditional / cycle

History: 1000; dimensions: 2; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 177166.7 | 178749.6 | 180027.5 | 5644.4 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 176451.1 | 179189.3 | 181259.3 | 5667.3 | 0/4 |
| parzen/full (scalar-f64) | supported | 25137.3 | 25448.0 | 26130.5 | 39781.6 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 24377.2 | 24599.7 | 25044.4 | 41022.0 | 4/4 |

## conditional / quality

History: 1000; dimensions: 2; budget: 25.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.250000 | 0.031622 | 0.250000 | 12.5% | 0.250000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.031622 | 0.250000 | 12.5% | 0.250000 |
| parzen/full (scalar-f64) | supported | 32 | 0.250000 | 0.031622 | 0.250000 | 12.5% | 0.250000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.031622 | 0.250000 | 12.5% | 0.250000 |

## conditional / quality

History: 1000; dimensions: 2; budget: 50.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.250000 | 0.000683 | 0.250000 | 18.8% | 0.250000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.000683 | 0.250000 | 18.8% | 0.250000 |
| parzen/full (scalar-f64) | supported | 32 | 0.250000 | 0.000683 | 0.250000 | 18.8% | 0.250000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.000683 | 0.250000 | 18.8% | 0.250000 |

## conditional / quality

History: 1000; dimensions: 2; budget: 100.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.250000 | 0.000130 | 0.250000 | 31.2% | 0.250000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.000130 | 0.250000 | 31.2% | 0.250000 |
| parzen/full (scalar-f64) | supported | 32 | 0.250000 | 0.000130 | 0.250000 | 31.2% | 0.250000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.250000 | 0.000130 | 0.250000 | 31.2% | 0.250000 |

## conditional / quality

History: 1000; dimensions: 2; budget: 250.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.031199 | 0.000000 | 0.250000 | 46.9% | 0.031199 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.031199 | 0.000000 | 0.250000 | 46.9% | 0.031199 |
| parzen/full (scalar-f64) | supported | 32 | 0.031199 | 0.000000 | 0.250000 | 46.9% | 0.031199 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.031199 | 0.000000 | 0.250000 | 46.9% | 0.031199 |

## conditional / suggest

History: 1000; dimensions: 2; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 87198.9 | 87594.8 | 88255.9 | 11468.0 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 86676.8 | 87012.6 | 91169.7 | 11537.1 | 0/4 |
| parzen/full (scalar-f64) | supported | 388.7 | 391.1 | 394.0 | 2572618.5 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 377.1 | 380.1 | 384.8 | 2651505.7 | 4/4 |

## correlated-numeric / cycle

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 602518.4 | 607131.4 | 617763.9 | 1659.7 | 0/4 |
| parzen/bounded (scalar-f64) | supported | 394546.9 | 398877.2 | 400561.8 | 2534.6 | 4/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 397732.1 | 400711.5 | 405535.2 | 2514.3 | 0/4 |
| parzen/full (scalar-f64) | supported | 761213.6 | 765342.2 | 771537.5 | 1313.7 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 764163.2 | 770297.0 | 775381.7 | 1308.6 | 0/4 |

## correlated-numeric / quality

History: 1000; dimensions: 4; budget: 25.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 146.099762 | 42.585723 | 366.496064 | 0.0% | 146.099762 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 146.099762 | 42.585723 | 366.496064 | 0.0% | 146.099762 |
| parzen/full (scalar-f64) | supported | 32 | 146.099762 | 42.585723 | 366.496064 | 0.0% | 146.099762 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 146.099762 | 42.585723 | 366.496064 | 0.0% | 146.099762 |

## correlated-numeric / quality

History: 1000; dimensions: 4; budget: 50.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 38.665840 | 9.470001 | 100.480363 | 0.0% | 38.665840 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 38.665840 | 9.470001 | 100.480363 | 0.0% | 38.665840 |
| parzen/full (scalar-f64) | supported | 32 | 38.665840 | 9.470001 | 100.480363 | 0.0% | 38.665840 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 38.665840 | 9.470001 | 100.480363 | 0.0% | 38.665840 |

## correlated-numeric / quality

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 7.513469 | 3.716869 | 26.282546 | 0.0% | 7.513469 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 7.513469 | 3.716869 | 26.282546 | 0.0% | 7.513469 |
| parzen/full (scalar-f64) | supported | 32 | 7.513469 | 3.716869 | 26.282546 | 0.0% | 7.513469 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 7.513469 | 3.716869 | 26.282546 | 0.0% | 7.513469 |

## correlated-numeric / quality

History: 1000; dimensions: 4; budget: 250.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 2.989532 | 1.953479 | 5.466015 | 0.0% | 2.989532 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 2.989532 | 1.953479 | 5.466015 | 0.0% | 2.989532 |
| parzen/full (scalar-f64) | supported | 32 | 2.989532 | 1.953479 | 5.466015 | 0.0% | 2.989532 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 2.989532 | 1.953479 | 5.466015 | 0.0% | 2.989532 |

## correlated-numeric / suggest

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 564313.6 | 568597.3 | 571614.1 | 1772.1 | 0/4 |
| parzen/bounded (scalar-f64) | supported | 180217.4 | 181390.0 | 183305.0 | 5548.9 | 4/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 182029.1 | 183264.7 | 185277.6 | 5493.6 | 0/4 |
| parzen/full (scalar-f64) | supported | 353480.7 | 355875.9 | 357974.1 | 2829.0 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 358156.3 | 359955.6 | 362170.9 | 2792.1 | 0/4 |

## independent-float / cycle

History: 10; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 61411.1 | 62457.6 | 63897.4 | 16283.7 | 1/4 |
| parzen/bounded (scalar-f64) | supported | 71478.5 | 72191.0 | 74630.9 | 13990.2 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 57083.3 | 57919.6 | 76127.2 | 17518.3 | 3/4 |
| parzen/full (scalar-f64) | supported | 66517.8 | 67561.4 | 68028.7 | 15033.6 | 0/4 |

## independent-float / cycle

History: 100; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 172445.0 | 174071.8 | 176523.4 | 5798.9 | 0/4 |
| parzen/bounded (scalar-f64) | supported | 195417.6 | 197433.7 | 198933.1 | 5117.2 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 156313.0 | 157944.0 | 160383.9 | 6397.4 | 4/4 |
| parzen/full (scalar-f64) | supported | 180446.3 | 182067.6 | 184536.8 | 5541.8 | 0/4 |

## independent-float / cycle

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 152502.2 | 154817.6 | 155627.6 | 6557.3 | 2/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 153137.5 | 154545.2 | 156083.1 | 6530.1 | 2/4 |
| parzen/full (scalar-f64) | supported | 283744.5 | 285967.3 | 287917.7 | 3524.3 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 283657.8 | 286198.1 | 289823.8 | 3525.4 | 0/4 |

## independent-float / cycle

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| hyperopt | supported | 1700384.8 | 1712929.6 | 1756903.1 | 588.1 | 0/4 |
| optimizer | supported | 712181.9 | 723036.3 | 726589.6 | 1404.1 | 1/4 |
| parzen/bounded (pulp-avx2-fma) | supported | 717537.2 | 720889.3 | 737762.8 | 1393.7 | 3/4 |
| parzen/bounded (scalar-f64) | supported | 778832.8 | 785197.6 | 790369.2 | 1284.0 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 1359965.6 | 1367311.3 | 1388620.7 | 735.3 | 0/4 |
| parzen/full (scalar-f64) | supported | 1483768.7 | 1492385.6 | 1527943.2 | 674.0 | 0/4 |
| tpe | supported | 892223.4 | 899804.9 | 905817.0 | 1120.8 | 0/4 |

## independent-float / cycle

History: 1000; dimensions: 8; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 1468735.1 | 1481497.1 | 1483930.4 | 680.9 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 1582446.0 | 1592745.5 | 1601515.0 | 631.9 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 2856833.1 | 2876603.6 | 2909556.8 | 350.0 | 0/4 |
| parzen/full (scalar-f64) | supported | 3113091.6 | 3135955.5 | 3154660.9 | 321.2 | 0/4 |

## independent-float / cycle

History: 1000; dimensions: 16; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 3104607.9 | 3119760.0 | 3136278.5 | 322.1 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 3297414.1 | 3321496.1 | 3331859.2 | 303.3 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 5853366.2 | 5906979.3 | 5940582.4 | 170.8 | 0/4 |
| parzen/full (scalar-f64) | supported | 6312011.8 | 6367173.0 | 6407954.0 | 158.4 | 0/4 |

## independent-float / cycle

History: 10000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 6925568.1 | 6989883.6 | 7047300.1 | 144.4 | 0/8 |
| parzen/bounded (pulp-avx2-fma) | supported | 708023.7 | 724012.2 | 730283.1 | 1412.4 | 8/8 |
| parzen/bounded (scalar-f64) | supported | 773690.1 | 782683.1 | 791301.1 | 1292.5 | 0/8 |
| parzen/full (pulp-avx2-fma) | supported | 14364002.4 | 14794326.4 | 15094789.2 | 69.6 | 0/8 |
| parzen/full (scalar-f64) | supported | 16264294.3 | 16374342.3 | 16442540.8 | 61.5 | 0/8 |

## independent-float / cycle

History: 100000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 708507.9 | 714479.9 | 718425.9 | 1411.4 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 772710.9 | 777659.9 | 786283.6 | 1294.1 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 160298472.0 | 161338864.0 | 162156921.0 | 6.2 | 0/4 |
| parzen/full (scalar-f64) | supported | 175398583.0 | 177818291.0 | 179620379.0 | 5.7 | 0/4 |

## independent-float / memory

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Retained after ingest | Retained at cycle end | Cycle bytes/op | Peak live bytes | Peak RSS bytes |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 1247168 | 1616144 | 147658.7 | 1633762 | 6160384 |
| parzen/bounded (pulp-avx2-fma) | supported | 205264 | 463464 | 1546.6 | 463530 | 4325376 |
| parzen/full (pulp-avx2-fma) | supported | 197928 | 856600 | 121411.4 | 873050 | 4718592 |

## independent-float / profile

Profile workload: `cycle`.

History: 10000; dimensions: 4; budget: 100.

| Backend | Status | Workload | Operations | Start observations | End observations | Profile seconds |
|---|---|---|---:|---:|---:|---:|
| parzen/full (pulp-avx2-fma) | supported | cycle | 2052 | 10000 | 12052 | 30.013 |
| parzen/full (scalar-f64) | supported | cycle | 1837 | 10000 | 11837 | 30.002 |

## independent-float / profile

Profile workload: `fixed-suggest`.

History: 10000; dimensions: 4; budget: 100.

| Backend | Status | Workload | Operations | Start observations | End observations | Profile seconds |
|---|---|---|---:|---:|---:|---:|
| parzen/full (pulp-avx2-fma) | supported | fixed-suggest | 2739 | 10000 | 10000 | 30.009 |
| parzen/full (scalar-f64) | supported | fixed-suggest | 2401 | 10000 | 10000 | 30.001 |

## independent-float / quality

History: 1000; dimensions: 4; budget: 25.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 32 | 17.319447 | 6.845352 | 33.206629 | 0.0% | 17.319447 |
| parzen/bounded (scalar-f64) | supported | 32 | 17.319447 | 6.845352 | 33.206629 | 0.0% | 17.319447 |
| parzen/full (pulp-avx2-fma) | supported | 32 | 17.319447 | 6.845352 | 33.206629 | 0.0% | 17.319447 |
| parzen/full (scalar-f64) | supported | 32 | 17.319447 | 6.845352 | 33.206629 | 0.0% | 17.319447 |

## independent-float / quality

History: 1000; dimensions: 4; budget: 50.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 32 | 8.326971 | 3.448432 | 12.132915 | 0.0% | 8.326971 |
| parzen/bounded (scalar-f64) | supported | 32 | 8.326971 | 3.448432 | 12.132915 | 0.0% | 8.326971 |
| parzen/full (pulp-avx2-fma) | supported | 32 | 8.326971 | 3.448432 | 12.132915 | 0.0% | 8.326971 |
| parzen/full (scalar-f64) | supported | 32 | 8.326971 | 3.448432 | 12.132915 | 0.0% | 8.326971 |

## independent-float / quality

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 32 | 1.320619 | 0.592668 | 3.134938 | 0.0% | 1.320619 |
| parzen/bounded (scalar-f64) | supported | 32 | 1.320619 | 0.592668 | 3.134938 | 0.0% | 1.320619 |
| parzen/full (pulp-avx2-fma) | supported | 32 | 1.320619 | 0.592668 | 3.134938 | 0.0% | 1.320619 |
| parzen/full (scalar-f64) | supported | 32 | 1.320619 | 0.592668 | 3.134938 | 0.0% | 1.320619 |

## independent-float / quality

History: 1000; dimensions: 4; budget: 250.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 32 | 0.135747 | 0.038941 | 0.404591 | 0.0% | 0.135747 |
| parzen/bounded (scalar-f64) | supported | 32 | 0.135747 | 0.038941 | 0.404591 | 0.0% | 0.135747 |
| parzen/full (pulp-avx2-fma) | supported | 32 | 0.135747 | 0.038941 | 0.404591 | 0.0% | 0.135747 |
| parzen/full (scalar-f64) | supported | 32 | 0.135747 | 0.038941 | 0.404591 | 0.0% | 0.135747 |

## independent-float / suggest

History: 10; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 14238.3 | 14310.7 | 14521.3 | 70233.3 | 3/4 |
| parzen/bounded (scalar-f64) | supported | 15278.6 | 15340.0 | 15654.0 | 65451.1 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 14239.6 | 14451.9 | 15453.3 | 70226.6 | 1/4 |
| parzen/full (scalar-f64) | supported | 15243.4 | 15345.4 | 16495.7 | 65602.0 | 0/4 |

## independent-float / suggest

History: 100; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 75504.8 | 76109.0 | 77915.8 | 13244.2 | 3/4 |
| parzen/bounded (scalar-f64) | supported | 83520.5 | 85210.5 | 85840.0 | 11973.1 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 75670.3 | 76383.9 | 77485.3 | 13215.2 | 1/4 |
| parzen/full (scalar-f64) | supported | 82784.1 | 84038.9 | 85067.7 | 12079.6 | 0/4 |

## independent-float / suggest

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 93469.8 | 94008.7 | 96892.8 | 10698.6 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 92367.0 | 93248.1 | 94015.2 | 10826.4 | 4/4 |
| parzen/full (scalar-f64) | supported | 184799.6 | 185745.2 | 186612.0 | 5411.3 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 183677.7 | 185254.2 | 187067.7 | 5444.3 | 0/4 |

## independent-float / suggest

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| hyperopt | supported | 1673395.8 | 1683004.9 | 1686829.2 | 597.6 | 0/4 |
| optimizer | supported | 692237.3 | 695993.9 | 702195.1 | 1444.6 | 0/4 |
| parzen/bounded (pulp-avx2-fma) | supported | 481617.2 | 487356.6 | 490529.4 | 2076.3 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 547460.6 | 553908.2 | 568155.4 | 1826.6 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 990929.0 | 998562.5 | 1003356.9 | 1009.2 | 0/4 |
| parzen/full (scalar-f64) | supported | 1120459.0 | 1132728.4 | 1140434.3 | 892.5 | 0/4 |
| tpe | supported | 806554.7 | 820455.4 | 829234.1 | 1239.8 | 0/4 |

## independent-float / suggest

History: 1000; dimensions: 8; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 1042933.2 | 1050755.9 | 1054152.8 | 958.8 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 1173040.2 | 1179873.2 | 1185734.5 | 852.5 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 2107722.5 | 2117633.4 | 2152862.5 | 474.4 | 0/4 |
| parzen/full (scalar-f64) | supported | 2349687.9 | 2366907.2 | 2378481.7 | 425.6 | 0/4 |

## independent-float / suggest

History: 1000; dimensions: 16; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 2180623.6 | 2190969.5 | 2204653.6 | 458.6 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 2382101.5 | 2412218.4 | 2419218.2 | 419.8 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 4351632.5 | 4369474.7 | 4394107.3 | 229.8 | 0/4 |
| parzen/full (scalar-f64) | supported | 4760444.8 | 4813834.7 | 4859459.5 | 210.1 | 0/4 |

## independent-float / suggest

History: 10000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 6936260.0 | 7004171.3 | 7073710.8 | 144.2 | 0/8 |
| parzen/bounded (pulp-avx2-fma) | supported | 457467.6 | 464203.5 | 484545.3 | 2185.9 | 8/8 |
| parzen/bounded (scalar-f64) | supported | 518344.4 | 528616.1 | 531807.7 | 1929.2 | 0/8 |
| parzen/full (pulp-avx2-fma) | supported | 10744006.4 | 10849150.5 | 10989980.3 | 93.1 | 0/8 |
| parzen/full (scalar-f64) | supported | 12331726.5 | 12430965.9 | 12475666.2 | 81.1 | 0/8 |

## independent-float / suggest

History: 100000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (pulp-avx2-fma) | supported | 439589.2 | 444751.7 | 447377.1 | 2274.9 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 497589.6 | 505606.6 | 510975.2 | 2009.7 | 0/4 |
| parzen/full (pulp-avx2-fma) | supported | 159317766.0 | 160898185.0 | 161961029.0 | 6.3 | 0/4 |
| parzen/full (scalar-f64) | supported | 176165939.0 | 177628357.0 | 179059850.0 | 5.7 | 0/4 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 146160.1 | 147361.0 | 148967.8 | 6841.8 | 4/4 |
| parzen/bounded (scalar-f64) | supported | 525896.3 | 528877.5 | 532362.2 | 1901.5 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 524693.5 | 527637.0 | 540139.3 | 1905.9 | 0/4 |
| parzen/full (scalar-f64) | supported | 643336.9 | 648480.4 | 651709.7 | 1554.4 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 644077.4 | 650658.8 | 655884.2 | 1552.6 | 0/4 |

## integer / memory

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Retained after ingest | Retained at cycle end | Cycle bytes/op | Peak live bytes | Peak RSS bytes |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 780922 | 1102784 | 41108.7 | 1120306 | 4980736 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 85608 | 175448 | 753.3 | 175464 | 3932160 |
| parzen/full (scalar-f64-policy-fallback) | supported | 114832 | 319840 | 30891.4 | 336240 | 4194304 |

## integer / quality

History: 1000; dimensions: 1; budget: 25.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 1.000000 | 0.000000 | 9.000000 | 21.9% | 1.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 1.000000 | 0.000000 | 9.000000 | 21.9% | 1.000000 |
| parzen/full (scalar-f64) | supported | 32 | 1.000000 | 0.000000 | 9.000000 | 21.9% | 1.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 1.000000 | 0.000000 | 9.000000 | 21.9% | 1.000000 |

## integer / quality

History: 1000; dimensions: 1; budget: 50.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 1.000000 | 62.5% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 1.000000 | 62.5% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 1.000000 | 62.5% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 1.000000 | 62.5% | 0.000000 |

## integer / quality

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## integer / quality

History: 1000; dimensions: 1; budget: 250.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 140804.2 | 142418.6 | 143635.1 | 7102.1 | 0/4 |
| parzen/bounded (scalar-f64) | supported | 1768.8 | 1780.8 | 1961.1 | 565340.9 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 1624.5 | 1629.7 | 1645.7 | 615580.2 | 0/4 |
| parzen/full (scalar-f64) | supported | 1588.8 | 1600.5 | 1677.9 | 629391.0 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 1516.6 | 1523.1 | 1532.5 | 659383.8 | 4/4 |

## linear-float / quality

History: 1000; dimensions: 1; budget: 25.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.007072 | 0.000246 | 0.065781 | 62.5% | 0.007072 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.007072 | 0.000246 | 0.065781 | 62.5% | 0.007072 |
| parzen/full (scalar-f64) | supported | 32 | 0.007072 | 0.000246 | 0.065781 | 62.5% | 0.007072 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.007072 | 0.000246 | 0.065781 | 62.5% | 0.007072 |

## linear-float / quality

History: 1000; dimensions: 1; budget: 50.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000806 | 0.000020 | 0.004241 | 96.9% | 0.000806 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000806 | 0.000020 | 0.004241 | 96.9% | 0.000806 |
| parzen/full (scalar-f64) | supported | 32 | 0.000806 | 0.000020 | 0.004241 | 96.9% | 0.000806 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000806 | 0.000020 | 0.004241 | 96.9% | 0.000806 |

## linear-float / quality

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000043 | 0.000001 | 0.000431 | 100.0% | 0.000043 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000043 | 0.000001 | 0.000431 | 100.0% | 0.000043 |
| parzen/full (scalar-f64) | supported | 32 | 0.000043 | 0.000001 | 0.000431 | 100.0% | 0.000043 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000043 | 0.000001 | 0.000431 | 100.0% | 0.000043 |

## linear-float / quality

History: 1000; dimensions: 1; budget: 250.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000002 | 0.000000 | 0.000015 | 100.0% | 0.000002 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000002 | 0.000000 | 0.000015 | 100.0% | 0.000002 |
| parzen/full (scalar-f64) | supported | 32 | 0.000002 | 0.000000 | 0.000015 | 100.0% | 0.000002 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000002 | 0.000000 | 0.000015 | 100.0% | 0.000002 |

## log-float / cycle

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 155839.7 | 157646.6 | 158787.6 | 6416.9 | 3/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 156471.5 | 158068.0 | 160253.4 | 6390.9 | 1/4 |
| parzen/full (scalar-f64) | supported | 289506.7 | 293266.7 | 294987.3 | 3454.2 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 287534.0 | 290029.6 | 292246.6 | 3477.8 | 0/4 |

## log-float / quality

History: 1000; dimensions: 1; budget: 25.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.004469 | 0.000187 | 0.046946 | 56.2% | 0.004469 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.004469 | 0.000187 | 0.046946 | 56.2% | 0.004469 |
| parzen/full (scalar-f64) | supported | 32 | 0.004469 | 0.000187 | 0.046946 | 56.2% | 0.004469 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.004469 | 0.000187 | 0.046946 | 56.2% | 0.004469 |

## log-float / quality

History: 1000; dimensions: 1; budget: 50.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000540 | 0.000042 | 0.002479 | 100.0% | 0.000540 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000540 | 0.000042 | 0.002479 | 100.0% | 0.000540 |
| parzen/full (scalar-f64) | supported | 32 | 0.000540 | 0.000042 | 0.002479 | 100.0% | 0.000540 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000540 | 0.000042 | 0.002479 | 100.0% | 0.000540 |

## log-float / quality

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000047 | 0.000002 | 0.000185 | 100.0% | 0.000047 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000047 | 0.000002 | 0.000185 | 100.0% | 0.000047 |
| parzen/full (scalar-f64) | supported | 32 | 0.000047 | 0.000002 | 0.000185 | 100.0% | 0.000047 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000047 | 0.000002 | 0.000185 | 100.0% | 0.000047 |

## log-float / quality

History: 1000; dimensions: 1; budget: 250.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000001 | 0.000000 | 0.000010 | 100.0% | 0.000001 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000001 | 0.000000 | 0.000010 | 100.0% | 0.000001 |
| parzen/full (scalar-f64) | supported | 32 | 0.000001 | 0.000000 | 0.000010 | 100.0% | 0.000001 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000001 | 0.000000 | 0.000010 | 100.0% | 0.000001 |

## log-float / suggest

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 97248.8 | 97913.4 | 98551.4 | 10282.9 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 96892.7 | 97269.0 | 98210.5 | 10320.7 | 4/4 |
| parzen/full (scalar-f64) | supported | 192019.1 | 194000.1 | 195625.0 | 5207.8 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 191373.4 | 192390.4 | 193120.4 | 5225.4 | 0/4 |

## mixed-independent / cycle

History: 1000; dimensions: 3; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 787363.1 | 796085.0 | 799468.6 | 1270.1 | 3/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 790840.5 | 796347.8 | 801908.1 | 1264.5 | 1/4 |
| parzen/full (scalar-f64) | supported | 1407983.2 | 1416888.5 | 1423703.3 | 710.2 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 1406931.0 | 1418707.4 | 1429773.4 | 710.8 | 0/4 |

## mixed-independent / quality

History: 1000; dimensions: 3; budget: 25.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 1.907527 | 1.071005 | 4.831794 | 0.0% | 1.907527 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 1.907527 | 1.071005 | 4.831794 | 0.0% | 1.907527 |
| parzen/full (scalar-f64) | supported | 32 | 1.907527 | 1.071005 | 4.831794 | 0.0% | 1.907527 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 1.907527 | 1.071005 | 4.831794 | 0.0% | 1.907527 |

## mixed-independent / quality

History: 1000; dimensions: 3; budget: 50.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 1.186190 | 0.462368 | 2.722823 | 0.0% | 1.186190 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 1.186190 | 0.462368 | 2.722823 | 0.0% | 1.186190 |
| parzen/full (scalar-f64) | supported | 32 | 1.186190 | 0.462368 | 2.722823 | 0.0% | 1.186190 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 1.186190 | 0.462368 | 2.722823 | 0.0% | 1.186190 |

## mixed-independent / quality

History: 1000; dimensions: 3; budget: 100.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 1.007674 | 0.078161 | 1.096562 | 0.0% | 1.007674 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 1.007674 | 0.078161 | 1.096562 | 0.0% | 1.007674 |
| parzen/full (scalar-f64) | supported | 32 | 1.007674 | 0.078161 | 1.096562 | 0.0% | 1.007674 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 1.007674 | 0.078161 | 1.096562 | 0.0% | 1.007674 |

## mixed-independent / quality

History: 1000; dimensions: 3; budget: 250.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.368568 | 0.000826 | 1.004520 | 28.1% | 0.368568 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.368568 | 0.000826 | 1.004520 | 28.1% | 0.368568 |
| parzen/full (scalar-f64) | supported | 32 | 0.368568 | 0.000826 | 1.004520 | 28.1% | 0.368568 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.368568 | 0.000826 | 1.004520 | 28.1% | 0.368568 |

## mixed-independent / suggest

History: 1000; dimensions: 3; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 110591.1 | 112278.6 | 113462.4 | 9042.3 | 2/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 111335.3 | 112128.1 | 124425.8 | 8981.9 | 2/4 |
| parzen/full (scalar-f64) | supported | 233946.9 | 234760.7 | 236876.0 | 4274.5 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 234242.5 | 235344.2 | 236388.8 | 4269.1 | 0/4 |

## stepped-integer / cycle

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 304022.1 | 306873.0 | 310424.6 | 3289.2 | 3/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 305664.7 | 308768.5 | 312495.3 | 3271.6 | 1/4 |
| parzen/full (scalar-f64) | supported | 312795.6 | 314667.9 | 316750.6 | 3197.0 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 311641.1 | 317969.3 | 319833.1 | 3208.8 | 0/4 |

## stepped-integer / quality

History: 1000; dimensions: 1; budget: 25.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 25.000000 | 84.4% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 25.000000 | 84.4% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 25.000000 | 84.4% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 25.000000 | 84.4% | 0.000000 |

## stepped-integer / quality

History: 1000; dimensions: 1; budget: 50.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## stepped-integer / quality

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## stepped-integer / quality

History: 1000; dimensions: 1; budget: 250.

| Backend | Status | Seeds | Median regret | p10 regret | p90 regret | Success ≤ 0.01 | Median best |
|---|---|---:|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |
| parzen/full (scalar-f64-policy-fallback) | supported | 32 | 0.000000 | 0.000000 | 0.000000 | 100.0% | 0.000000 |

## stepped-integer / suggest

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| parzen/bounded (scalar-f64) | supported | 1440.5 | 1452.7 | 1502.7 | 694191.5 | 0/4 |
| parzen/bounded (scalar-f64-policy-fallback) | supported | 1400.2 | 1406.7 | 1419.8 | 714170.8 | 0/4 |
| parzen/full (scalar-f64) | supported | 1353.7 | 1360.5 | 1363.7 | 738726.5 | 0/4 |
| parzen/full (scalar-f64-policy-fallback) | supported | 1323.3 | 1332.0 | 1351.1 | 755679.4 | 4/4 |
