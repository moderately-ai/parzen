# TPE comparison results

> Absolute timings are machine-specific. Compare only runs captured on the same machine. Timing and quality are reported independently; no combined score is calculated.

## categorical / cycle

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 36486.9 | 36819.4 | 38888.2 | 27407.1 | 0/3 |
| parzen/bounded | supported | 32263.3 | 32413.4 | 32977.4 | 30995.0 | 2/3 |
| parzen/full | supported | 32276.1 | 32492.2 | 33690.4 | 30982.7 | 1/3 |

## categorical / suggest

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 33672.2 | 34830.6 | 35299.4 | 29698.1 | 0/3 |
| parzen/bounded | supported | 478.0 | 482.4 | 485.4 | 2091967.2 | 1/3 |
| parzen/full | supported | 478.5 | 481.7 | 484.8 | 2090072.5 | 2/3 |

## correlated-numeric / cycle

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 602516.9 | 606120.1 | 610594.7 | 1659.7 | 0/3 |
| parzen/bounded | supported | 372864.2 | 374571.4 | 378876.9 | 2681.9 | 3/3 |
| parzen/full | supported | 744520.5 | 750176.5 | 752593.4 | 1343.1 | 0/3 |

## correlated-numeric / suggest

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 565827.6 | 568233.7 | 574856.8 | 1767.3 | 0/3 |
| parzen/bounded | supported | 155516.9 | 155872.1 | 156573.0 | 6430.2 | 3/3 |
| parzen/full | supported | 306713.1 | 308286.7 | 310506.8 | 3260.4 | 0/3 |

## independent-float / cycle

History: 10; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 40970.1 | 41653.9 | 41831.8 | 24408.0 | 3/3 |
| parzen/bounded | supported | 71382.6 | 72005.9 | 72472.4 | 14009.0 | 0/3 |
| parzen/full | supported | 68204.7 | 68499.6 | 69396.0 | 14661.8 | 0/3 |

## independent-float / cycle

History: 100; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 96845.4 | 98021.4 | 98920.5 | 10325.7 | 3/3 |
| parzen/bounded | supported | 193779.0 | 195732.0 | 197061.5 | 5160.5 | 0/3 |
| parzen/full | supported | 182007.1 | 183280.4 | 184099.5 | 5494.3 | 0/3 |

## independent-float / cycle

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 143662.0 | 145024.1 | 147355.3 | 6960.8 | 3/3 |
| parzen/bounded | supported | 152938.9 | 153908.6 | 155506.2 | 6538.6 | 0/3 |
| parzen/full | supported | 283445.0 | 285746.5 | 286752.5 | 3528.0 | 0/3 |

## independent-float / cycle

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 707286.4 | 715035.0 | 717794.2 | 1413.9 | 3/3 |
| parzen/bounded | supported | 764640.8 | 770222.5 | 777468.9 | 1307.8 | 0/3 |
| parzen/full | supported | 1486067.9 | 1493536.1 | 1497587.3 | 672.9 | 0/3 |

## independent-float / cycle

History: 1000; dimensions: 8; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 1688548.6 | 1696425.7 | 1706338.8 | 592.2 | 0/3 |
| parzen/bounded | supported | 1553532.7 | 1563468.0 | 1569740.6 | 643.7 | 3/3 |
| parzen/full | supported | 3114860.5 | 3147405.9 | 3158373.0 | 321.0 | 0/3 |

## independent-float / cycle

History: 1000; dimensions: 16; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 4369754.0 | 4420006.1 | 4478805.0 | 228.8 | 0/3 |
| parzen/bounded | supported | 3220998.8 | 3236793.9 | 3260182.3 | 310.5 | 3/3 |
| parzen/full | supported | 6358669.6 | 6374136.8 | 6439999.0 | 157.3 | 0/3 |

## independent-float / cycle

History: 10000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 6841459.7 | 6894157.6 | 6916196.1 | 146.2 | 0/3 |
| parzen/bounded | supported | 756042.1 | 762595.1 | 766330.0 | 1322.7 | 3/3 |
| parzen/full | supported | 16297589.4 | 16409172.9 | 16607571.2 | 61.4 | 0/3 |

## independent-float / cycle

History: 100000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 134567458.0 | 135278054.0 | 142179435.0 | 7.4 | 0/3 |
| parzen/bounded | supported | 748446.0 | 756965.4 | 760708.8 | 1336.1 | 3/3 |
| parzen/full | supported | 180879403.0 | 182050571.0 | 182998824.0 | 5.5 | 0/3 |

## independent-float / memory

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Retained after ingest | Retained at cycle end | Cycle bytes/op | Peak live bytes | Peak RSS bytes |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 1247168 | 1616144 | 147658.7 | 1633762 | 6029312 |
| parzen/bounded | supported | 200496 | 519104 | 1546.6 | 519170 | 4325376 |
| parzen/full | supported | 191944 | 1002784 | 122068.8 | 1019234 | 4718592 |

## independent-float / suggest

History: 10; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 10210.9 | 10280.3 | 10356.1 | 97934.9 | 3/3 |
| parzen/bounded | supported | 16317.5 | 16440.5 | 16921.1 | 61284.1 | 0/3 |
| parzen/full | supported | 16315.9 | 16411.4 | 16501.8 | 61289.8 | 0/3 |

## independent-float / suggest

History: 100; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 65512.8 | 65653.8 | 66443.1 | 15264.2 | 3/3 |
| parzen/bounded | supported | 79138.8 | 79801.3 | 81721.4 | 12636.0 | 0/3 |
| parzen/full | supported | 79173.6 | 79960.4 | 80218.3 | 12630.5 | 0/3 |

## independent-float / suggest

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 132127.9 | 133337.8 | 135628.4 | 7568.4 | 0/3 |
| parzen/bounded | supported | 85429.1 | 86341.5 | 88107.8 | 11705.6 | 3/3 |
| parzen/full | supported | 169586.4 | 171017.5 | 172466.2 | 5896.7 | 0/3 |

## independent-float / suggest

History: 1000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 683308.1 | 689433.4 | 693086.4 | 1463.5 | 0/3 |
| parzen/bounded | supported | 520224.5 | 528105.2 | 532995.8 | 1922.2 | 3/3 |
| parzen/full | supported | 1070823.8 | 1079630.7 | 1113614.8 | 933.9 | 0/3 |

## independent-float / suggest

History: 1000; dimensions: 8; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 1635672.1 | 1643523.4 | 1651328.5 | 611.4 | 0/3 |
| parzen/bounded | supported | 1125176.4 | 1132641.1 | 1140602.8 | 888.7 | 3/3 |
| parzen/full | supported | 2237125.5 | 2265449.3 | 2296332.4 | 447.0 | 0/3 |

## independent-float / suggest

History: 1000; dimensions: 16; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 4324620.3 | 4357923.3 | 4377517.7 | 231.2 | 0/3 |
| parzen/bounded | supported | 2297121.3 | 2310833.4 | 2324159.9 | 435.3 | 3/3 |
| parzen/full | supported | 4578931.3 | 4635481.2 | 4689988.4 | 218.4 | 0/3 |

## independent-float / suggest

History: 10000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 6905718.6 | 6942222.1 | 7041095.2 | 144.8 | 0/3 |
| parzen/bounded | supported | 496650.3 | 502943.0 | 509697.2 | 2013.5 | 3/3 |
| parzen/full | supported | 12197934.0 | 12253169.4 | 12637561.4 | 82.0 | 0/3 |

## independent-float / suggest

History: 100000; dimensions: 4; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 134348743.0 | 135241144.0 | 138876394.0 | 7.4 | 0/3 |
| parzen/bounded | supported | 477514.2 | 483490.2 | 486617.6 | 2094.2 | 3/3 |
| parzen/full | supported | 180662038.0 | 181337945.0 | 182215085.0 | 5.5 | 0/3 |

## integer / cycle

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 144432.7 | 145130.8 | 146596.1 | 6923.6 | 3/8 |
| parzen/bounded | supported | 519528.0 | 523977.4 | 526922.3 | 1924.8 | 5/8 |
| parzen/full | supported | 646197.9 | 651744.6 | 656256.9 | 1547.5 | 0/8 |

## integer / memory

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Retained after ingest | Retained at cycle end | Cycle bytes/op | Peak live bytes | Peak RSS bytes |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 780922 | 1102784 | 41108.7 | 1120306 | 4980736 |
| parzen/bounded | supported | 84416 | 187844 | 753.3 | 187860 | 3932160 |
| parzen/full | supported | 104832 | 343892 | 31048.2 | 360292 | 4194304 |

## integer / suggest

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 138728.8 | 139944.6 | 141031.1 | 7208.3 | 0/8 |
| parzen/bounded | supported | 1706.0 | 1719.6 | 1765.2 | 586175.1 | 8/8 |
| parzen/full | supported | 1750.5 | 1762.2 | 1778.8 | 571276.9 | 0/8 |

## log-float / cycle

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 147295.5 | 148895.6 | 150288.0 | 6789.1 | 3/3 |
| parzen/bounded | supported | 155872.5 | 156937.9 | 164199.3 | 6415.5 | 0/3 |
| parzen/full | supported | 287300.6 | 289790.1 | 291133.1 | 3480.7 | 0/3 |

## log-float / suggest

History: 1000; dimensions: 1; budget: 100.

| Backend | Status | Min ns/op | Median ns/op | p95 ns/op | Ops/s | Round wins |
|---|---|---:|---:|---:|---:|---:|
| optimizer | supported | 137012.6 | 138342.6 | 139868.2 | 7298.6 | 0/3 |
| parzen/bounded | supported | 89766.8 | 90407.1 | 91191.7 | 11140.0 | 3/3 |
| parzen/full | supported | 179361.6 | 180459.3 | 182726.3 | 5575.3 | 0/3 |
