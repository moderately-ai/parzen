#!/usr/bin/env bash
set -euo pipefail

repo_dir=${1:-/home/tsanterre/workspace/github.com/moderately-ai/parzen-comparison-harness}
output_dir=${2:-comparison-benchmarks/results/raw/counters}
suite=${3:-all}
counter_seconds=${COUNTER_SECONDS:-10}
cpu=${COUNTER_CPU:-7}
original_paranoid=$(sysctl -n kernel.perf_event_paranoid)
changed=0

restore_host() {
  if (( changed )); then
    sudo sysctl "kernel.perf_event_paranoid=${original_paranoid}" >/dev/null
  fi
  final_paranoid=$(sysctl -n kernel.perf_event_paranoid)
  printf 'restored_utc=%s\nfinal_perf_event_paranoid=%s\nrestoration_ok=%s\n' \
    "$(date -u +%FT%TZ)" "$final_paranoid" "$([[ "$final_paranoid" == "$original_paranoid" ]] && echo true || echo false)" \
    >>"$audit_log"
  [[ "$final_paranoid" == "$original_paranoid" ]]
}

cd "$repo_dir"
mkdir -p "$output_dir"
audit_log="$output_dir/host-change.log"
trap restore_host EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

{
  printf 'started_utc=%s\ncommit=%s\ndirty=%s\n' "$(date -u +%FT%TZ)" "$(git rev-parse HEAD)" "$(git status --porcelain | wc -l)"
  printf 'original_perf_event_paranoid=%s\ncpu=%s\ngovernor=%s\naffinity=%s\n' \
    "$original_paranoid" "$cpu" "$(cat "/sys/devices/system/cpu/cpu${cpu}/cpufreq/scaling_governor")" "$(taskset -pc $$)"
  printf 'rust=%s\nperf=%s\nload=%s\n' "$(rustc --version)" "$(perf --version)" "$(uptime)"
  ps -Ao pcpu,pid,comm --sort=-pcpu | head -17 || true
} >"$audit_log"

sudo sysctl kernel.perf_event_paranoid=1 >/dev/null
changed=1
printf 'changed_utc=%s\nchanged_perf_event_paranoid=%s\n' "$(date -u +%FT%TZ)" "$(sysctl -n kernel.perf_event_paranoid)" >>"$audit_log"

CARGO_TARGET_DIR=comparison-benchmarks/target-calibration \
  RUSTFLAGS='-C target-cpu=native' \
  cargo build --manifest-path comparison-benchmarks/Cargo.toml --release --bin calibrate-machine
cargo build --manifest-path comparison-benchmarks/Cargo.toml --profile profiling --bins

calibration_bin=comparison-benchmarks/target-calibration/release/calibrate-machine
profiling_dir=comparison-benchmarks/target/profiling
sha256sum "$calibration_bin" "$profiling_dir"/bench-{parzen,optimizer} >>"$audit_log"

run_group() {
  local label=$1
  local events=$2
  shift 2
  for repetition in 1 2 3 4 5; do
    local csv="$output_dir/${label}.r${repetition}.csv"
    local json="$output_dir/${label}.r${repetition}.json"
    set +e
    taskset -c "$cpu" perf stat --no-big-num -x, -r 1 -e "$events" \
      -o "$csv" -- "$@" >"$json"
    local status=$?
    set -e
    if (( status != 0 )); then
      rm -f "$csv" "$json"
      printf 'failed_counter_group=%s repetition=%s events=%s exit_status=%s\n' \
        "$label" "$repetition" "$events" "$status" >>"$audit_log"
      break
    fi
  done
}

if [[ "$suite" == all || "$suite" == calibration ]]; then
  run_group calibration-fma-core cycles,instructions,branches,branch-misses \
    "$calibration_bin" fma --seconds "$counter_seconds" --format json
  run_group calibration-fma-flops fp_ret_sse_avx_ops.all \
    "$calibration_bin" fma --seconds "$counter_seconds" --format json
  run_group calibration-bandwidth-core cycles,instructions,cache-references,cache-misses \
    "$calibration_bin" bandwidth --bytes 268435456 --seconds "$counter_seconds" --format json
  run_group calibration-bandwidth-dram nps1_die_to_dram \
    "$calibration_bin" bandwidth --bytes 268435456 --seconds "$counter_seconds" --format json
fi

run_case() {
  local label=$1
  local binary=$2
  local history_mode=$3
  local scenario=$4
  local history=$5
  local dimensions=$6
  local workload=$7
  shift 7
  local command=("$profiling_dir/$binary" --scenario "$scenario" --operation profile \
    --profile-workload "$workload" --history "$history" --dimensions "$dimensions" \
    --profile-seconds "$counter_seconds" --parzen-history "$history_mode" \
    --machine-label balthasar-5800x-cpu7 --format json)
  run_group "${label}-core" cycles,instructions "${command[@]}"
  run_group "${label}-branch" branches,branch-misses "${command[@]}"
  run_group "${label}-cache" cache-references,cache-misses "${command[@]}"
  run_group "${label}-l2" l2_request_g1.all_no_prefetch,l2_cache_req_stat.ic_dc_miss_in_l2 \
    "${command[@]}"
  run_group "${label}-l3-proxy" l2_pf_miss_l2_hit_l3,l2_pf_miss_l2_l3 \
    "${command[@]}"
  run_group "${label}-dram" nps1_die_to_dram "${command[@]}"
  run_group "${label}-os" context-switches,cpu-migrations,page-faults "${command[@]}"
  run_group "${label}-flops" fp_ret_sse_avx_ops.all "${command[@]}"
}

if [[ "$suite" == all || "$suite" == cases ]]; then
  for history in 1000 10000 100000; do
    for mode in full bounded; do
      run_case "float-h${history}-d4-parzen-${mode}-fixed" bench-parzen "$mode" independent-float "$history" 4 fixed-suggest
      run_case "float-h${history}-d4-parzen-${mode}-cycle" bench-parzen "$mode" independent-float "$history" 4 cycle
    done
    run_case "float-h${history}-d4-optimizer-fixed" bench-optimizer full independent-float "$history" 4 fixed-suggest
    run_case "float-h${history}-d4-optimizer-cycle" bench-optimizer full independent-float "$history" 4 cycle
  done

  for mode in full bounded; do
    run_case "float-h1000-d16-parzen-${mode}-fixed" bench-parzen "$mode" independent-float 1000 16 fixed-suggest
    run_case "integer-h1000-parzen-${mode}-fixed" bench-parzen "$mode" integer 1000 1 fixed-suggest
    run_case "integer-h1000-parzen-${mode}-cycle" bench-parzen "$mode" integer 1000 1 cycle
  done
  run_case float-h1000-d16-optimizer-fixed bench-optimizer full independent-float 1000 16 fixed-suggest
  run_case integer-h1000-optimizer-fixed bench-optimizer full integer 1000 1 fixed-suggest
  run_case integer-h1000-optimizer-cycle bench-optimizer full integer 1000 1 cycle
fi

printf 'completed_utc=%s\n' "$(date -u +%FT%TZ)" >>"$audit_log"
